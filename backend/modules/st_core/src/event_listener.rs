//! Soroban event listener — background worker that polls Horizon for on-chain
//! staking events and transitions matches from `AWAITING_STAKE` to `IN_PROGRESS`.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │  SorobanEventListener                                   │
//! │                                                         │
//! │  tracked_matches: HashMap<contract_id, TrackedMatch>    │
//! │       ↑ track_match() / untrack_match()                 │
//! │                                                         │
//! │  run() ──► tokio::time::interval(5s)                    │
//! │                 │                                       │
//! │                 ▼                                       │
//! │  poll_events() ──► GET /events?contract_id=...          │
//! │                 │                                       │
//! │                 ▼                                       │
//! │  is_stake_event() ──► true                              │
//! │                 │                                       │
//! │                 ▼                                       │
//! │  handle_stake_event()                                   │
//! │    status: AWAITING_STAKE → IN_PROGRESS                 │
//! │    broadcast::Sender<MatchStartSignal>::send()          │
//! │         │                                               │
//! └─────────┼───────────────────────────────────────────────┘
//!           │
//!           ▼
//!    WebSocket handler receives MatchStartSignal
//!    and forwards it to the connected client
//! ```

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};
use tokio::time::{interval, Duration};
use uuid::Uuid;

// ── Constants ─────────────────────────────────────────────────────────────────

/// How often the listener polls Horizon for new contract events.
/// 5 seconds gives a good balance between latency and API load.
const POLL_INTERVAL_SECS: u64 = 5;

/// Maximum events fetched per Horizon request.
const HORIZON_EVENT_LIMIT: u32 = 200;

/// Horizon event topic values that indicate a stake has been confirmed.
/// These must match the symbols your Soroban staking contract emits via
/// `env.events().publish(...)`.
///
/// Update this list if your contract uses different topic symbols.
const STAKE_TOPIC_SYMBOLS: &[&str] = &["staked", "stake_confirmed"];

// ── Public types ──────────────────────────────────────────────────────────────

/// The on-chain status of a match from the perspective of the event listener.
/// Mirrors the database `MatchStatus` enum — kept separate to avoid coupling
/// `st_core` to the database layer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StakeStatus {
    /// Waiting for both players to submit their XLM stake on-chain.
    AwaitingStake,
    /// Both stakes confirmed — game may begin.
    InProgress,
}

/// Signal broadcast to WebSocket subscribers when a match transitions to
/// `IN_PROGRESS` after its on-chain stake event is confirmed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchStartSignal {
    /// The XLMate match UUID.
    pub match_id: Uuid,
    /// New status — always "IN_PROGRESS" when this signal fires.
    pub status: String,
    /// The Soroban event type that triggered the transition.
    pub event_type: String,
    /// Stellar transaction hash of the staking transaction.
    pub transaction_hash: String,
    /// Ledger sequence number the event was recorded in.
    pub ledger: u32,
}

/// Internal record tracking a match that is waiting for its stake event.
#[derive(Debug)]
struct TrackedMatch {
    match_id: Uuid,
    contract_id: String,
    status: StakeStatus,
    /// Broadcast channel to notify WebSocket handlers when the stake arrives.
    notifier: broadcast::Sender<MatchStartSignal>,
}

// ── Horizon response shapes ───────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct HorizonEventsResponse {
    #[serde(rename = "_embedded")]
    embedded: EmbeddedEvents,
}

#[derive(Debug, Deserialize)]
struct EmbeddedEvents {
    records: Vec<HorizonEvent>,
}

/// A single event record from Horizon's `GET /events` endpoint.
#[derive(Debug, Deserialize)]
struct HorizonEvent {
    #[serde(rename = "type")]
    event_type: String,
    /// The Soroban contract that emitted this event.
    contract_id: Option<String>,
    /// Opaque cursor used to resume polling without reprocessing events.
    paging_token: String,
    /// List of base64-encoded XDR ScVal topics attached to the event.
    topic: Vec<String>,
    /// Ledger sequence number.
    ledger: u32,
    /// Stellar transaction hash.
    transaction_hash: String,
}

// ── Service ───────────────────────────────────────────────────────────────────

/// Background service that polls Horizon for Soroban staking events and
/// signals the WebSocket layer when a match's stake is confirmed on-chain.
///
/// # Usage
///
/// ```rust,no_run
/// use std::sync::Arc;
/// use st_core::event_listener::SorobanEventListener;
///
/// #[tokio::main]
/// async fn main() {
///     let listener = Arc::new(SorobanEventListener::new(
///         "https://horizon-testnet.stellar.org",
///     ));
///
///     // Register a match to watch
///     let mut rx = listener
///         .track_match(match_id, "CONTRACT_ID_HERE")
///         .await;
///
///     // Spawn the polling loop as a background task
///     let listener_clone = Arc::clone(&listener);
///     tokio::spawn(async move { listener_clone.run().await });
///
///     // Wait for the stake confirmed signal
///     if let Ok(signal) = rx.recv().await {
///         println!("Match {} is now IN_PROGRESS", signal.match_id);
///     }
/// }
/// ```
pub struct SorobanEventListener {
    /// Base URL of the Stellar Horizon instance to poll.
    horizon_url: String,
    /// Matches currently waiting for their on-chain stake event.
    /// Keyed by the Soroban contract ID associated with the match.
    tracked_matches: Arc<RwLock<HashMap<String, TrackedMatch>>>,
    /// Horizon event pagination cursor. Starts at "now" so we only process
    /// events that arrive after the listener starts, then advances with each
    /// batch so events are never processed twice.
    cursor: Arc<RwLock<String>>,
    /// Reusable HTTP client — keeps a connection pool alive across polls.
    http_client: reqwest::Client,
}

impl SorobanEventListener {
    /// Create a new listener pointed at `horizon_url`.
    ///
    /// Use `"https://horizon-testnet.stellar.org"` for testnet or
    /// `"https://horizon.stellar.org"` for mainnet.
    pub fn new(horizon_url: impl Into<String>) -> Self {
        Self {
            horizon_url: horizon_url.into(),
            tracked_matches: Arc::new(RwLock::new(HashMap::new())),
            cursor: Arc::new(RwLock::new("now".to_string())),
            http_client: reqwest::Client::new(),
        }
    }

    /// Register a match to watch for its stake confirmation event.
    ///
    /// Returns a `broadcast::Receiver<MatchStartSignal>` the caller should pass
    /// to the WebSocket handler. When the stake is confirmed on-chain, a
    /// `MatchStartSignal` is sent on this channel.
    ///
    /// # Arguments
    ///
    /// * `match_id`    — The XLMate UUID for this match.
    /// * `contract_id` — The Soroban contract ID that will emit the stake event.
    pub async fn track_match(
        &self,
        match_id: Uuid,
        contract_id: impl Into<String>,
    ) -> broadcast::Receiver<MatchStartSignal> {
        let (tx, rx) = broadcast::channel(16);
        let contract_id = contract_id.into();

        let tracked = TrackedMatch {
            match_id,
            contract_id: contract_id.clone(),
            status: StakeStatus::AwaitingStake,
            notifier: tx,
        };

        let mut matches = self.tracked_matches.write().await;
        matches.insert(contract_id.clone(), tracked);

        log::info!(
            "[EventListener] Tracking match {} on contract {}",
            match_id,
            contract_id
        );

        rx
    }

    /// Stop watching a match — call after the match starts or is cancelled.
    pub async fn untrack_match(&self, contract_id: &str) {
        let mut matches = self.tracked_matches.write().await;
        if matches.remove(contract_id).is_some() {
            log::info!("[EventListener] Untracked contract {}", contract_id);
        }
    }

    /// Start the polling loop. Spawn this in a dedicated Tokio task:
    ///
    /// ```rust,no_run
    /// tokio::spawn(async move { listener.run().await });
    /// ```
    ///
    /// The loop runs indefinitely. Errors from individual poll cycles are logged
    /// but do not stop the loop — transient Horizon outages are tolerated.
    pub async fn run(&self) {
        log::info!(
            "[EventListener] Started — polling {} every {}s",
            self.horizon_url,
            POLL_INTERVAL_SECS
        );

        // tokio::time::interval fires immediately on the first tick, then every
        // POLL_INTERVAL_SECS — equivalent to a JavaScript setInterval.
        let mut ticker = interval(Duration::from_secs(POLL_INTERVAL_SECS));

        loop {
            ticker.tick().await;

            if let Err(e) = self.poll_events().await {
                log::error!("[EventListener] Poll cycle failed: {}", e);
                // Continue — transient errors (network blip, Horizon 503) should
                // not kill the listener. The next tick will retry.
            }
        }
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    /// One poll cycle: fetch new events from Horizon for every tracked contract
    /// and process any that look like stake confirmations.
    async fn poll_events(&self) -> Result<()> {
        // Snapshot the tracked contracts to avoid holding the lock during I/O
        let contract_ids: Vec<String> = {
            let matches = self.tracked_matches.read().await;
            if matches.is_empty() {
                return Ok(()); // Nothing to watch yet
            }
            matches.keys().cloned().collect()
        };

        let cursor = self.cursor.read().await.clone();

        for contract_id in contract_ids {
            let url = format!(
                "{}/events?type=contract&contract_id={}&cursor={}&limit={}",
                self.horizon_url, contract_id, cursor, HORIZON_EVENT_LIMIT
            );

            let response = self.http_client.get(&url).send().await?;

            if !response.status().is_success() {
                log::warn!(
                    "[EventListener] Horizon returned {} for contract {}",
                    response.status(),
                    contract_id
                );
                continue;
            }

            let body: HorizonEventsResponse = response.json().await?;

            for event in body.embedded.records {
                // Always advance the cursor, even for non-stake events,
                // so we never reprocess the same ledger twice.
                {
                    let mut cursor_guard = self.cursor.write().await;
                    *cursor_guard = event.paging_token.clone();
                }

                if self.is_stake_event(&event, &contract_id) {
                    self.handle_stake_event(&contract_id, &event).await;
                }
            }
        }

        Ok(())
    }

    /// Return `true` if this Horizon event is a stake confirmation for the
    /// given contract.
    ///
    /// Soroban events encode topics as base64 XDR ScVal. This function decodes
    /// the first topic and checks whether it contains one of the stake symbols
    /// defined in `STAKE_TOPIC_SYMBOLS`. Adjust those constants to match
    /// whatever your Soroban contract emits.
    fn is_stake_event(&self, event: &HorizonEvent, expected_contract: &str) -> bool {
        if event.event_type != "contract" {
            return false;
        }

        // Verify the event came from the contract we are watching
        if event.contract_id.as_deref() != Some(expected_contract) {
            return false;
        }

        // Decode the first topic (ScVal symbol) and check it against our list
        event.topic.first().map_or(false, |topic_b64| {
            general_purpose::STANDARD
                .decode(topic_b64)
                .map(|bytes| {
                    let decoded = String::from_utf8_lossy(&bytes);
                    STAKE_TOPIC_SYMBOLS
                        .iter()
                        .any(|symbol| decoded.contains(symbol))
                })
                .unwrap_or(false)
        })
    }

    /// Process a confirmed stake event: transition the match status and
    /// broadcast the start signal to WebSocket subscribers.
    async fn handle_stake_event(&self, contract_id: &str, event: &HorizonEvent) {
        let mut matches = self.tracked_matches.write().await;

        let Some(tracked) = matches.get_mut(contract_id) else {
            return; // Race: match was untracked between poll and handle
        };

        if tracked.status != StakeStatus::AwaitingStake {
            // Already transitioned — ignore duplicate events from the same ledger
            return;
        }

        log::info!(
            "[EventListener] Stake confirmed — match {} (contract {}) tx {}. \
             Transitioning AWAITING_STAKE → IN_PROGRESS.",
            tracked.match_id,
            contract_id,
            event.transaction_hash
        );

        // Transition the in-memory status
        tracked.status = StakeStatus::InProgress;

        let signal = MatchStartSignal {
            match_id: tracked.match_id,
            status: "IN_PROGRESS".to_string(),
            event_type: "STAKE_CONFIRMED".to_string(),
            transaction_hash: event.transaction_hash.clone(),
            ledger: event.ledger,
        };

        match tracked.notifier.send(signal) {
            Ok(n) => log::info!(
                "[EventListener] Broadcast MatchStartSignal to {} WebSocket subscriber(s)",
                n
            ),
            Err(_) => log::warn!(
                "[EventListener] No active WebSocket subscribers for match {} — signal dropped",
                tracked.match_id
            ),
        }
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_listener() -> SorobanEventListener {
        SorobanEventListener::new("https://horizon-testnet.stellar.org")
    }

    fn make_event(contract_id: &str, topic_b64: &str, tx_hash: &str) -> HorizonEvent {
        HorizonEvent {
            event_type: "contract".to_string(),
            contract_id: Some(contract_id.to_string()),
            paging_token: "12345-1".to_string(),
            topic: vec![topic_b64.to_string()],
            ledger: 100,
            transaction_hash: tx_hash.to_string(),
        }
    }

    fn encode_topic(symbol: &str) -> String {
        general_purpose::STANDARD.encode(symbol.as_bytes())
    }

    // ── is_stake_event ────────────────────────────────────────────────────────

    #[test]
    fn test_stake_event_recognised_by_topic() {
        let listener = make_listener();
        let contract = "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4";
        let event = make_event(contract, &encode_topic("staked"), "txabc");

        assert!(listener.is_stake_event(&event, contract));
    }

    #[test]
    fn test_stake_confirmed_topic_recognised() {
        let listener = make_listener();
        let contract = "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4";
        let event = make_event(contract, &encode_topic("stake_confirmed"), "txabc");

        assert!(listener.is_stake_event(&event, contract));
    }

    #[test]
    fn test_non_stake_topic_ignored() {
        let listener = make_listener();
        let contract = "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4";
        // A different event emitted by the same contract (e.g. game_started)
        let event = make_event(contract, &encode_topic("game_started"), "txabc");

        assert!(!listener.is_stake_event(&event, contract));
    }

    #[test]
    fn test_wrong_contract_id_ignored() {
        let listener = make_listener();
        let contract  = "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4";
        let different = "CBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB";
        let event = make_event(different, &encode_topic("staked"), "txabc");

        assert!(!listener.is_stake_event(&event, contract));
    }

    #[test]
    fn test_non_contract_event_type_ignored() {
        let listener = make_listener();
        let contract = "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4";
        let mut event = make_event(contract, &encode_topic("staked"), "txabc");
        event.event_type = "diagnostic".to_string(); // Not a contract event

        assert!(!listener.is_stake_event(&event, contract));
    }

    #[test]
    fn test_invalid_base64_topic_ignored() {
        let listener = make_listener();
        let contract = "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4";
        let event = make_event(contract, "!!!not-valid-base64!!!", "txabc");

        // Should not panic — returns false gracefully
        assert!(!listener.is_stake_event(&event, contract));
    }

    // ── track_match / handle_stake_event ──────────────────────────────────────

    #[tokio::test]
    async fn test_track_match_registers_awaiting_stake_status() {
        let listener = make_listener();
        let match_id  = Uuid::new_v4();
        let contract  = "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4";

        listener.track_match(match_id, contract).await;

        let matches = listener.tracked_matches.read().await;
        let tracked = matches.get(contract).unwrap();
        assert_eq!(tracked.match_id, match_id);
        assert_eq!(tracked.status, StakeStatus::AwaitingStake);
    }

    #[tokio::test]
    async fn test_stake_event_broadcasts_start_signal() {
        let listener = make_listener();
        let match_id  = Uuid::new_v4();
        let contract  = "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4";

        let mut rx = listener.track_match(match_id, contract).await;

        let event = make_event(contract, &encode_topic("staked"), "tx_stake_hash");
        listener.handle_stake_event(contract, &event).await;

        let signal = rx.try_recv().expect("MatchStartSignal should be available");
        assert_eq!(signal.match_id, match_id);
        assert_eq!(signal.status, "IN_PROGRESS");
        assert_eq!(signal.event_type, "STAKE_CONFIRMED");
        assert_eq!(signal.transaction_hash, "tx_stake_hash");
        assert_eq!(signal.ledger, 100);
    }

    #[tokio::test]
    async fn test_stake_event_transitions_status_to_in_progress() {
        let listener = make_listener();
        let match_id  = Uuid::new_v4();
        let contract  = "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4";

        listener.track_match(match_id, contract).await;

        let event = make_event(contract, &encode_topic("staked"), "tx_hash");
        listener.handle_stake_event(contract, &event).await;

        let matches = listener.tracked_matches.read().await;
        assert_eq!(
            matches.get(contract).unwrap().status,
            StakeStatus::InProgress
        );
    }

    #[tokio::test]
    async fn test_duplicate_stake_event_ignored() {
        let listener = make_listener();
        let match_id  = Uuid::new_v4();
        let contract  = "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4";

        let mut rx = listener.track_match(match_id, contract).await;

        let event = make_event(contract, &encode_topic("staked"), "tx_hash");

        // First event — should broadcast
        listener.handle_stake_event(contract, &event).await;
        assert!(rx.try_recv().is_ok());

        // Second event for same match — should be a no-op (already IN_PROGRESS)
        listener.handle_stake_event(contract, &event).await;
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_untrack_removes_match() {
        let listener = make_listener();
        let match_id  = Uuid::new_v4();
        let contract  = "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4";

        listener.track_match(match_id, contract).await;
        listener.untrack_match(contract).await;

        let matches = listener.tracked_matches.read().await;
        assert!(matches.get(contract).is_none());
    }

    #[test]
    fn test_poll_interval_constant() {
        assert_eq!(POLL_INTERVAL_SECS, 5);
    }
}