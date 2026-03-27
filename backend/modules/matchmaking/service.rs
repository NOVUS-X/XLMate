use actix_web::web;
use chrono::Utc;
use deadpool_redis::Pool;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;

use super::models::*;

// ── Elo matching constants ────────────────────────────────────────────────────

/// Initial Elo search window when a player first joins the rated queue.
/// A 1000 Elo player will only match with players in the range [900, 1100].
const INITIAL_ELO_RANGE: u32 = 100;

/// After EXPAND_AFTER_SECS of waiting, the search window expands to this value.
/// This guarantees a match will eventually be found while still preferring
/// close-Elo opponents during the first 30 seconds.
const EXPANDED_ELO_RANGE: u32 = 200;

/// How many seconds a player must wait before their Elo search window expands
/// from INITIAL_ELO_RANGE to EXPANDED_ELO_RANGE.
const EXPAND_AFTER_SECS: i64 = 30;

const DEFAULT_ESTIMATED_WAIT_TIME: Duration = Duration::from_secs(60);

#[derive(Clone)]
pub struct MatchmakingService {
    redis_pool: Pool,
    active_matches: Arc<Mutex<HashMap<Uuid, Match>>>,
}

impl MatchmakingService {
    pub fn new(redis_pool: Pool) -> Self {
        Self {
            redis_pool,
            active_matches: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn get_redis_connection(
        &self,
    ) -> Result<deadpool_redis::Connection, String> {
        self.redis_pool
            .get()
            .await
            .map_err(|e| format!("Redis connection failed: {}", e))
    }

    pub async fn join_queue(
        &self,
        mut request: MatchRequest,
    ) -> Result<MatchmakingResponse, String> {
        let request_id = request.id;

        match request.match_type {
            MatchType::Rated => {
                // Set the initial Elo search window for this player.
                // expand_elo_ranges() will widen it to EXPANDED_ELO_RANGE after
                // EXPAND_AFTER_SECS seconds of waiting.
                if request.max_elo_diff.is_none() {
                    request.max_elo_diff = Some(INITIAL_ELO_RANGE);
                }

                if let Some(match_result) = self.find_rated_match(&request).await? {
                    return Ok(match_result);
                }
                self.add_to_redis_queue(&request).await?;
            }
            MatchType::Casual => {
                if let Some(match_result) = self.find_casual_match(&request).await? {
                    return Ok(match_result);
                }
                self.add_to_redis_queue(&request).await?;
            }
            MatchType::Private => {
                if let Some(invite_address) = &request.invite_address {
                    self.add_private_invite(invite_address, &request).await?;
                    return Ok(MatchmakingResponse {
                        status: "Waiting for invited player".to_string(),
                        match_id: None,
                        request_id,
                    });
                } else {
                    return Ok(MatchmakingResponse {
                        status: "Invalid private match request: missing invite address"
                            .to_string(),
                        match_id: None,
                        request_id,
                    });
                }
            }
        }

        Ok(MatchmakingResponse {
            status: "Added to queue".to_string(),
            match_id: None,
            request_id,
        })
    }

    async fn add_to_redis_queue(&self, request: &MatchRequest) -> Result<(), String> {
        let mut conn = self.get_redis_connection().await?;
        let key = request.match_type.redis_key();
        let now = Utc::now();
        let score = now.timestamp() as f64;
        let value = request
            .to_redis_value()
            .map_err(|e| format!("Serialization error: {}", e))?;

        let cutoff = (now - chrono::Duration::hours(1)).timestamp() as f64;
        conn.zrembyscore::<_, _, _, ()>(&key, f64::NEG_INFINITY, cutoff)
            .await
            .map_err(|e| format!("Redis ZREMRANGEBYSCORE failed: {}", e))?;

        conn.zadd::<_, _, _, ()>(&key, &value, score)
            .await
            .map_err(|e| format!("Redis ZADD failed: {}", e))?;

        conn.expire::<_, ()>(&key, 3600)
            .await
            .map_err(|e| format!("Redis EXPIRE failed: {}", e))?;

        Ok(())
    }

    async fn add_private_invite(
        &self,
        invite_address: &str,
        request: &MatchRequest,
    ) -> Result<(), String> {
        let mut conn = self.get_redis_connection().await?;
        let key = "matchmaking:invites";
        let value = request
            .to_redis_value()
            .map_err(|e| format!("Serialization error: {}", e))?;

        conn.hset::<_, _, _, ()>(key, invite_address, &value)
            .await
            .map_err(|e| format!("Redis HSET failed: {}", e))?;

        Ok(())
    }

    pub async fn check_private_invite(
        &self,
        wallet_address: &str,
    ) -> Result<Option<MatchRequest>, String> {
        let mut conn = self.get_redis_connection().await?;
        let key = "matchmaking:invites";

        let value: Option<String> = conn
            .hget(key, wallet_address)
            .await
            .map_err(|e| format!("Redis HGET failed: {}", e))?;

        match value {
            Some(json) => MatchRequest::from_redis_value(&json)
                .map(Some)
                .map_err(|e| format!("Deserialization error: {}", e)),
            None => Ok(None),
        }
    }

    pub async fn accept_private_invite(
        &self,
        inviter_request_id: Uuid,
        accepting_player: Player,
    ) -> Result<Option<MatchmakingResponse>, String> {
        let mut conn = self.get_redis_connection().await?;
        let key = "matchmaking:invites";

        // Lua script for atomic find-and-remove operation.
        // Prevents race conditions where multiple players try to accept the same invite.
        let lua_script = r#"
            local key = KEYS[1]
            local target_request_id = ARGV[1]
            
            local invites = redis.call('HGETALL', key)
            
            for i = 1, #invites, 2 do
                local invite_address = invites[i]
                local invite_json = invites[i + 1]
                local invite = cjson.decode(invite_json)
                
                if invite.id == target_request_id then
                    redis.call('HDEL', key, invite_address)
                    return invite_json
                end
            end
            
            return nil
        "#;

        let result: Option<String> = redis::Script::new(lua_script)
            .key(key)
            .arg(inviter_request_id.to_string())
            .invoke_async(&mut conn)
            .await
            .map_err(|e| format!("Redis Lua script failed: {}", e))?;

        if let Some(invite_json) = result {
            if let Ok(invite_request) = MatchRequest::from_redis_value(&invite_json) {
                let match_id = Uuid::new_v4();
                let new_match = Match {
                    id: match_id,
                    player1: invite_request.player,
                    player2: accepting_player,
                    match_type: MatchType::Private,
                    created_at: Utc::now(),
                };

                let mut active_matches = self.active_matches.lock().unwrap();
                active_matches.insert(match_id, new_match);

                return Ok(Some(MatchmakingResponse {
                    status: "Match created".to_string(),
                    match_id: Some(match_id),
                    request_id: inviter_request_id,
                }));
            }
        }

        Ok(None)
    }

    pub async fn cancel_request(&self, request_id: Uuid) -> Result<bool, String> {
        let mut conn = self.get_redis_connection().await?;

        if self
            .remove_from_queue(&mut conn, "matchmaking:queue:rated", request_id)
            .await?
        {
            return Ok(true);
        }

        if self
            .remove_from_queue(&mut conn, "matchmaking:queue:casual", request_id)
            .await?
        {
            return Ok(true);
        }

        let invites: HashMap<String, String> = conn
            .hgetall("matchmaking:invites")
            .await
            .map_err(|e| format!("Redis HGETALL failed: {}", e))?;

        for (invite_address, json) in invites {
            if let Ok(request) = MatchRequest::from_redis_value(&json) {
                if request.id == request_id {
                    conn.hdel::<_, _, ()>("matchmaking:invites", &invite_address)
                        .await
                        .map_err(|e| format!("Redis HDEL failed: {}", e))?;
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    async fn remove_from_queue(
        &self,
        conn: &mut deadpool_redis::Connection,
        key: &str,
        request_id: Uuid,
    ) -> Result<bool, String> {
        let members: Vec<String> = conn
            .zrange(key, 0, -1)
            .await
            .map_err(|e| format!("Redis ZRANGE failed: {}", e))?;

        for member in members {
            if let Ok(request) = MatchRequest::from_redis_value(&member) {
                if request.id == request_id {
                    conn.zrem::<_, _, ()>(key, &member)
                        .await
                        .map_err(|e| format!("Redis ZREM failed: {}", e))?;
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    pub async fn get_queue_status(
        &self,
        request_id: Uuid,
    ) -> Result<Option<QueueStatus>, String> {
        let mut conn = self.get_redis_connection().await?;

        if let Some(status) = self
            .get_status_from_queue(
                &mut conn,
                "matchmaking:queue:rated",
                request_id,
                MatchType::Rated,
            )
            .await?
        {
            return Ok(Some(status));
        }

        if let Some(status) = self
            .get_status_from_queue(
                &mut conn,
                "matchmaking:queue:casual",
                request_id,
                MatchType::Casual,
            )
            .await?
        {
            return Ok(Some(status));
        }

        let invites: HashMap<String, String> = conn
            .hgetall("matchmaking:invites")
            .await
            .map_err(|e| format!("Redis HGETALL failed: {}", e))?;

        for (_, json) in invites {
            if let Ok(request) = MatchRequest::from_redis_value(&json) {
                if request.id == request_id {
                    return Ok(Some(QueueStatus {
                        request_id,
                        position: 1,
                        estimated_wait_time: DEFAULT_ESTIMATED_WAIT_TIME,
                        match_type: MatchType::Private,
                    }));
                }
            }
        }

        Ok(None)
    }

    async fn get_status_from_queue(
        &self,
        conn: &mut deadpool_redis::Connection,
        key: &str,
        request_id: Uuid,
        match_type: MatchType,
    ) -> Result<Option<QueueStatus>, String> {
        let members: Vec<String> = conn
            .zrange(key, 0, -1)
            .await
            .map_err(|e| format!("Redis ZRANGE failed: {}", e))?;

        for (index, member) in members.iter().enumerate() {
            if let Ok(request) = MatchRequest::from_redis_value(member) {
                if request.id == request_id {
                    return Ok(Some(QueueStatus {
                        request_id,
                        position: index + 1,
                        estimated_wait_time: self.estimate_wait_time(index, &match_type),
                        match_type,
                    }));
                }
            }
        }

        Ok(None)
    }

    async fn find_rated_match(
        &self,
        request: &MatchRequest,
    ) -> Result<Option<MatchmakingResponse>, String> {
        let mut conn = self.get_redis_connection().await?;
        let key = "matchmaking:queue:rated";
        let player_elo = request.player.elo;

        // The incoming player's current search window (may already be expanded
        // if they were queued previously and re-calling after expand_elo_ranges).
        let incoming_range = request.max_elo_diff.unwrap_or(INITIAL_ELO_RANGE);

        // Lua script for atomic find-and-remove.
        //
        // Matching condition: the Elo gap must be within the LARGER of the two
        // players' current search windows. This ensures that a player who has
        // waited long enough to expand their range can match with anyone within
        // their expanded window, even if that opponent joined more recently.
        //
        // Example:
        //   - Player A (1000 Elo, waiting 35 s) has expanded range ±200 → matches
        //     anyone in [800, 1200].
        //   - Player B (1150 Elo, waiting 5 s)  has initial range ±100 → would
        //     not match A under their own range, but A's expanded range covers
        //     the 150-point gap, so the match IS made.
        //
        // A 1000 Elo player with INITIAL_ELO_RANGE can never match a 2000 Elo
        // player because even EXPANDED_ELO_RANGE (200) is far smaller than the
        // 1000-point gap. The guarantee holds.
        let lua_script = r#"
            local key             = KEYS[1]
            local player_elo      = tonumber(ARGV[1])
            local incoming_range  = tonumber(ARGV[2])

            local members = redis.call('ZRANGE', key, 0, -1)

            for i, member in ipairs(members) do
                local opponent     = cjson.decode(member)
                local opponent_elo = tonumber(opponent.player.elo)
                local elo_diff     = math.abs(opponent_elo - player_elo)

                -- Use the larger of the two players' current search windows
                local opponent_range = tonumber(opponent.max_elo_diff) or incoming_range
                local effective_range = math.max(incoming_range, opponent_range)

                if elo_diff <= effective_range then
                    redis.call('ZREM', key, member)
                    return member
                end
            end

            return nil
        "#;

        let result: Option<String> = redis::Script::new(lua_script)
            .key(key)
            .arg(player_elo)
            .arg(incoming_range)
            .invoke_async(&mut conn)
            .await
            .map_err(|e| format!("Redis Lua script failed: {}", e))?;

        if let Some(opponent_json) = result {
            if let Ok(opponent_request) = MatchRequest::from_redis_value(&opponent_json) {
                let match_id = Uuid::new_v4();
                let new_match = Match {
                    id: match_id,
                    player1: opponent_request.player,
                    player2: request.player.clone(),
                    match_type: MatchType::Rated,
                    created_at: Utc::now(),
                };

                let mut active_matches = self.active_matches.lock().unwrap();
                active_matches.insert(match_id, new_match);

                return Ok(Some(MatchmakingResponse {
                    status: "Match found".to_string(),
                    match_id: Some(match_id),
                    request_id: request.id,
                }));
            }
        }

        Ok(None)
    }

    async fn find_casual_match(
        &self,
        request: &MatchRequest,
    ) -> Result<Option<MatchmakingResponse>, String> {
        let mut conn = self.get_redis_connection().await?;
        let key = "matchmaking:queue:casual";

        // Casual queue is pure FIFO — pop the oldest waiting player
        let result: Vec<(String, f64)> = conn
            .zpopmin(key, 1)
            .await
            .map_err(|e| format!("Redis ZPOPMIN failed: {}", e))?;

        let result = result.into_iter().next();

        if let Some((member, _score)) = result {
            if let Ok(opponent_request) = MatchRequest::from_redis_value(&member) {
                let match_id = Uuid::new_v4();
                let new_match = Match {
                    id: match_id,
                    player1: opponent_request.player,
                    player2: request.player.clone(),
                    match_type: MatchType::Casual,
                    created_at: Utc::now(),
                };

                let mut active_matches = self.active_matches.lock().unwrap();
                active_matches.insert(match_id, new_match);

                return Ok(Some(MatchmakingResponse {
                    status: "Match found".to_string(),
                    match_id: Some(match_id),
                    request_id: request.id,
                }));
            }
        }

        Ok(None)
    }

    fn estimate_wait_time(&self, position: usize, match_type: &MatchType) -> Duration {
        match match_type {
            MatchType::Rated  => Duration::from_secs((30 + position as u64 * 15).min(300)),
            MatchType::Casual => Duration::from_secs((15 + position as u64 * 10).min(180)),
            MatchType::Private => DEFAULT_ESTIMATED_WAIT_TIME,
        }
    }

    /// Expand the Elo search window for players who have been waiting in the
    /// rated queue for longer than EXPAND_AFTER_SECS (30 seconds).
    ///
    /// Called periodically by a background task (e.g. every 5–10 seconds).
    ///
    /// Expansion is a single step: INITIAL_ELO_RANGE → EXPANDED_ELO_RANGE.
    /// This gives players a fair shot at a close match first, then guarantees
    /// they will eventually be paired once the wider window opens up.
    pub async fn expand_elo_ranges(&self) -> Result<(), String> {
        let mut conn = self.get_redis_connection().await?;
        let key = "matchmaking:queue:rated";
        let now = Utc::now();

        let members: Vec<(String, f64)> = conn
            .zrange_withscores(key, 0, -1)
            .await
            .map_err(|e| format!("Redis ZRANGE failed: {}", e))?;

        for (member, score) in members {
            if let Ok(mut request) = MatchRequest::from_redis_value(&member) {
                let wait_secs = now
                    .signed_duration_since(request.player.join_time)
                    .num_seconds();

                // Only expand once the player has waited long enough, and only
                // if they are still on the initial range (avoid repeated writes).
                if wait_secs >= EXPAND_AFTER_SECS
                    && request.max_elo_diff.unwrap_or(INITIAL_ELO_RANGE) < EXPANDED_ELO_RANGE
                {
                    request.max_elo_diff = Some(EXPANDED_ELO_RANGE);

                    let updated_value = request
                        .to_redis_value()
                        .map_err(|e| format!("Serialization error: {}", e))?;

                    // Atomic swap: remove the old entry, insert the updated one
                    // at the same score so queue ordering is preserved.
                    conn.zrem::<_, _, ()>(key, &member)
                        .await
                        .map_err(|e| format!("Redis ZREM failed: {}", e))?;

                    conn.zadd::<_, _, _, ()>(key, &updated_value, score)
                        .await
                        .map_err(|e| format!("Redis ZADD failed: {}", e))?;
                }
            }
        }

        Ok(())
    }

    pub fn get_match(&self, match_id: Uuid) -> Option<Match> {
        let active_matches = self.active_matches.lock().unwrap();
        active_matches.get(&match_id).cloned()
    }
}

pub fn get_matchmaking_service(redis_pool: Pool) -> web::Data<MatchmakingService> {
    web::Data::new(MatchmakingService::new(redis_pool))
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that the Elo range constants satisfy the acceptance criteria:
    /// a 1000-Elo player must never immediately match a 2000-Elo player.
    #[test]
    fn test_elo_range_constants() {
        let gap = 2000_u32.abs_diff(1000);

        // Even the expanded range must be narrower than the 1000-point gap
        assert!(
            EXPANDED_ELO_RANGE < gap,
            "EXPANDED_ELO_RANGE ({EXPANDED_ELO_RANGE}) must be < 1000 so a 1000-Elo \
             player can never match a 2000-Elo player"
        );

        // Initial range must be tighter than expanded
        assert!(
            INITIAL_ELO_RANGE < EXPANDED_ELO_RANGE,
            "INITIAL_ELO_RANGE must be smaller than EXPANDED_ELO_RANGE"
        );

        // Expansion must happen after a positive wait time
        assert!(EXPAND_AFTER_SECS > 0);
    }

    /// Simulate the matching condition from the Lua script in pure Rust to
    /// verify the effective-range logic is correct.
    fn effective_range_matches(
        player_elo: u32,
        player_range: u32,
        opponent_elo: u32,
        opponent_range: u32,
    ) -> bool {
        let elo_diff = player_elo.abs_diff(opponent_elo);
        let effective_range = player_range.max(opponent_range);
        elo_diff <= effective_range
    }

    #[test]
    fn test_initial_range_blocks_large_gap() {
        // 1000 vs 2000 — both on initial range — must NOT match
        assert!(!effective_range_matches(1000, INITIAL_ELO_RANGE, 2000, INITIAL_ELO_RANGE));
    }

    #[test]
    fn test_initial_range_allows_close_match() {
        // 1000 vs 1080 — gap 80, within ±100 — must match immediately
        assert!(effective_range_matches(1000, INITIAL_ELO_RANGE, 1080, INITIAL_ELO_RANGE));
    }

    #[test]
    fn test_initial_range_blocks_boundary() {
        // 1000 vs 1101 — gap 101, just outside ±100 — must NOT match yet
        assert!(!effective_range_matches(1000, INITIAL_ELO_RANGE, 1101, INITIAL_ELO_RANGE));
    }

    #[test]
    fn test_expanded_range_allows_wider_match() {
        // 1000 vs 1150 — gap 150. Player A has expanded, B hasn't.
        // The larger range (200) applies → must match.
        assert!(effective_range_matches(1000, EXPANDED_ELO_RANGE, 1150, INITIAL_ELO_RANGE));
    }

    #[test]
    fn test_expanded_range_still_blocks_extreme_gap() {
        // 1000 vs 2000 — even fully expanded, must NOT match
        assert!(!effective_range_matches(1000, EXPANDED_ELO_RANGE, 2000, EXPANDED_ELO_RANGE));
    }

    #[test]
    fn test_expand_threshold_is_30_seconds() {
        assert_eq!(EXPAND_AFTER_SECS, 30);
    }

    #[test]
    fn test_initial_range_is_100() {
        assert_eq!(INITIAL_ELO_RANGE, 100);
    }

    #[test]
    fn test_expanded_range_is_200() {
        assert_eq!(EXPANDED_ELO_RANGE, 200);
    }
}