use serde::{Deserialize, Serialize};
use std::env;
use std::fmt;

/// Errors that can occur during PGN archival
#[derive(Debug)]
pub enum PgnArchiveError {
    IpfsError(String),
    ArweaveError(String),
    SerializationError(String),
    NetworkError(String),
}

impl fmt::Display for PgnArchiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IpfsError(e) => write!(f, "IPFS error: {}", e),
            Self::ArweaveError(e) => write!(f, "Arweave error: {}", e),
            Self::SerializationError(e) => write!(f, "Serialization error: {}", e),
            Self::NetworkError(e) => write!(f, "Network error: {}", e),
        }
    }
}

impl std::error::Error for PgnArchiveError {}

/// Response from IPFS pinning
#[derive(Debug, Serialize, Deserialize)]
pub struct IpfsResponse {
    #[serde(rename = "IpfsHash")]
    pub ipfs_hash: String,
    #[serde(rename = "PinSize")]
    pub pin_size: usize,
    #[serde(rename = "Timestamp")]
    pub timestamp: String,
}

/// Response from Arweave upload
#[derive(Debug, Serialize, Deserialize)]
pub struct ArweaveResponse {
    pub id: String,
    pub status: u16,
}

/// Archive result containing both IPFS and Arweave identifiers
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PgnArchiveResult {
    pub game_id: String,
    pub ipfs_cid: Option<String>,
    pub arweave_tx_id: Option<String>,
    pub pgn_hash: String,
    pub archived_at: String,
}

/// PGN Archival Service for storing game data on decentralized storage
#[derive(Clone, Debug)]
pub struct PgnArchiveService {
    client: reqwest::Client,
}

impl PgnArchiveService {
    pub fn new() -> Self {
        PgnArchiveService {
            client: reqwest::Client::new(),
        }
    }

    /// Archive PGN to IPFS
    pub async fn archive_to_ipfs(&self, pgn_content: &str) -> Result<String, PgnArchiveError> {
        let ipfs_url = env::var("IPFS_API_URL")
            .unwrap_or_else(|_| "https://api.pinata.cloud/pinning/pinJSONToIPFS".to_string());
        
        let ipfs_api_key = env::var("IPFS_API_KEY")
            .unwrap_or_else(|_| "".to_string());
        
        let ipfs_secret_key = env::var("IPFS_SECRET_KEY")
            .unwrap_or_else(|_| "".to_string());

        let payload = serde_json::json!({
            "pinataContent": {
                "pgn": pgn_content,
                "type": "chess_game",
                "archived_at": chrono::Utc::now().to_rfc3339()
            },
            "pinataMetadata": {
                "name": format!("xlmate_game_{}.pgn", chrono::Utc::now().timestamp())
            }
        });

        let response = self.client
            .post(&ipfs_url)
            .header("Content-Type", "application/json")
            .header("pinata_api_key", &ipfs_api_key)
            .header("pinata_secret_api_key", &ipfs_secret_key)
            .json(&payload)
            .send()
            .await
            .map_err(|e| PgnArchiveError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(PgnArchiveError::IpfsError(
                format!("IPFS upload failed with status: {}", response.status())
            ));
        }

        let ipfs_response: IpfsResponse = response
            .json()
            .await
            .map_err(|e| PgnArchiveError::SerializationError(e.to_string()))?;

        Ok(ipfs_response.ipfs_hash)
    }

    /// Archive PGN to Arweave
    pub async fn archive_to_arweave(&self, pgn_content: &str) -> Result<String, PgnArchiveError> {
        let arweave_url = env::var("ARWEAVE_GATEWAY_URL")
            .unwrap_or_else(|_| "https://arweave.net".to_string());
        
        // For simplicity, we're using a bundler service
        // In production, you'd use the Arweave SDK with a wallet
        let bundler_url = env::var("ARWEAVE_BUNDLER_URL")
            .unwrap_or_else(|_| "https://node2.bundlr.network".to_string());

        // Calculate SHA-256 hash of PGN for integrity
        let pgn_hash = self.calculate_hash(pgn_content);

        let payload = serde_json::json!({
            "data": pgn_content,
            "tags": [
                { "name": "Content-Type", "value": "application/x-chess-pgn" },
                { "name": "App", "value": "XLMate" },
                { "name": "PGN-Hash", "value": &pgn_hash },
                { "name": "Archived-At", "value": &chrono::Utc::now().to_rfc3339() }
            ]
        });

        let response = self.client
            .post(&format!("{}/tx", bundler_url))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| PgnArchiveError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(PgnArchiveError::ArweaveError(
                format!("Arweave upload failed with status: {}", response.status())
            ));
        }

        let arweave_response: ArweaveResponse = response
            .json()
            .await
            .map_err(|e| PgnArchiveError::SerializationError(e.to_string()))?;

        Ok(arweave_response.id)
    }

    /// Archive PGN to both IPFS and Arweave
    pub async fn archive_pgn(
        &self,
        game_id: &str,
        pgn_content: &str,
    ) -> Result<PgnArchiveResult, PgnArchiveError> {
        let pgn_hash = self.calculate_hash(pgn_content);
        
        // Archive to both services in parallel
        let ipfs_future = self.archive_to_ipfs(pgn_content);
        let arweave_future = self.archive_to_arweave(pgn_content);

        let (ipfs_result, arweave_result) = tokio::join!(ipfs_future, arweave_future);

        let ipfs_cid = match ipfs_result {
            Ok(cid) => {
                log::info!("PGN archived to IPFS: {}", cid);
                Some(cid)
            }
            Err(e) => {
                log::warn!("Failed to archive to IPFS: {}", e);
                None
            }
        };

        let arweave_tx_id = match arweave_result {
            Ok(tx_id) => {
                log::info!("PGN archived to Arweave: {}", tx_id);
                Some(tx_id)
            }
            Err(e) => {
                log::warn!("Failed to archive to Arweave: {}", e);
                None
            }
        };

        if ipfs_cid.is_none() && arweave_tx_id.is_none() {
            return Err(PgnArchiveError::NetworkError(
                "Failed to archive PGN to any storage service".to_string()
            ));
        }

        Ok(PgnArchiveResult {
            game_id: game_id.to_string(),
            ipfs_cid,
            arweave_tx_id,
            pgn_hash,
            archived_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Calculate SHA-256 hash of PGN content
    fn calculate_hash(&self, content: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Verify PGN integrity by comparing hash
    pub fn verify_pgn_integrity(&self, pgn_content: &str, expected_hash: &str) -> bool {
        let actual_hash = self.calculate_hash(pgn_content);
        actual_hash == expected_hash
    }
}
