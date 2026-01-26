use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MatchType {
    Rated,
    Casual,
    Private,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub wallet_address: String,
    pub elo: u32,
    pub join_time: DateTime<Utc>, 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchRequest {
    pub id: Uuid,
    pub player: Player,
    pub match_type: MatchType,
    pub invite_address: Option<String>, // For private matches__
    pub max_elo_diff: Option<u32>,      // For rated matches__
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Match {
    pub id: Uuid,
    pub player1: Player,
    pub player2: Player,
    pub match_type: MatchType,
    pub created_at: DateTime<Utc>, 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchmakingQueue {
    pub rated_queue: Vec<MatchRequest>,
    pub casual_queue: Vec<MatchRequest>,
    pub private_invites: HashMap<String, MatchRequest>, // wallet_address -> request
}

impl MatchmakingQueue {
    pub fn new() -> Self {
        Self {
            rated_queue: Vec::new(),
            casual_queue: Vec::new(),
            private_invites: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStatus {
    pub request_id: Uuid,
    pub position: usize,
    pub estimated_wait_time: Duration,
    pub match_type: MatchType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchmakingResponse {
    pub status: String,
    pub match_id: Option<Uuid>,
    pub request_id: Uuid,
}