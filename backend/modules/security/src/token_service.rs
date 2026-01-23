//! Refresh token rotation lives here, using "Token Families" for theft detection.
//! Reusing a token? The whole family gets invalidated - that's how we catch thieves.

use chrono::{Duration, Utc};
use db::db::db::get_db;
use entity::refresh_token::{self, ActiveModel, Entity as RefreshToken};
use error::error::ApiError;
use rand::Rng;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, sea_query::Expr};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Token settings loaded from env vars, with sensible defaults.
pub struct TokenConfig {
    pub refresh_token_ttl_days: i64,
    pub account_lock_duration_minutes: i64,
    pub jwt_secret: String,
}

impl TokenConfig {
    pub fn from_env() -> Self {
        dotenv::dotenv().ok();
        
        let refresh_token_ttl_days = std::env::var("REFRESH_TOKEN_TTL_DAYS")
            .unwrap_or_else(|_| "7".to_string())
            .parse::<i64>()
            .unwrap_or(7);
            
        let account_lock_duration_minutes = std::env::var("ACCOUNT_LOCK_DURATION_MINUTES")
            .unwrap_or_else(|_| "30".to_string())
            .parse::<i64>()
            .unwrap_or(30);
            
        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| {
                // I enforce security in production - no default secrets allowed!
                #[cfg(not(debug_assertions))]
                panic!("FATAL: JWT_SECRET environment variable must be set in production mode!");
                
                // Only allow default in development/debug mode
                "default-secret-change-me".to_string()
            });
            
        Self {
            refresh_token_ttl_days,
            account_lock_duration_minutes,
            jwt_secret,
        }
    }
}

/// What gets returned after a successful token rotation.
pub struct TokenRotationResult {
    pub new_token: String,
    pub token_id: Uuid,
    pub expires_at: chrono::DateTime<Utc>,
    pub player_id: Uuid,
}

/// What the token verification found - valid, stolen, expired, etc.
pub enum TokenVerificationResult {
    /// Fresh, unused token - good to go
    Valid {
        player_id: Uuid,
        family_id: Uuid,
        token_id: Uuid,
    },
    /// Someone reused this token - possible theft!
    Reused {
        family_id: Uuid,
        player_id: Uuid,
    },
    /// Past its expiry date
    Expired,
    /// Not in the database
    NotFound,
    /// Explicitly revoked
    Revoked,
}

/// Creates a 32-byte random token, base64-encoded for URLs.
pub fn generate_token() -> String {
    use rand::RngCore;
    let mut rng = rand::thread_rng();
    let mut token_bytes = [0u8; 32];
    rng.fill_bytes(&mut token_bytes);
    base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, token_bytes)
}

/// SHA256 hashes the token - we never store tokens in plaintext.
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let result = hasher.finalize();
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, result)
}

/// Creates a fresh token with a new family - used on login.
pub async fn create_refresh_token(player_id: Uuid) -> Result<TokenRotationResult, ApiError> {
    let config = TokenConfig::from_env();
    create_refresh_token_with_family(player_id, Uuid::new_v4(), &config).await
}

/// Creates a token in an existing family - used during rotation.
pub async fn create_refresh_token_with_family(
    player_id: Uuid,
    family_id: Uuid,
    config: &TokenConfig,
) -> Result<TokenRotationResult, ApiError> {
    let db = get_db().await;
    
    let token = generate_token();
    let token_hash = hash_token(&token);
    let now = Utc::now();
    let expires_at = now + Duration::days(config.refresh_token_ttl_days);
    let token_id = Uuid::new_v4();
    
    // Use raw SQL to ensure UUID format consistency with player table
    // All UUIDs stored as hyphenated strings
    use sea_orm::{Statement, ConnectionTrait, Value, DbBackend};
    
    let sql = "INSERT INTO refresh_tokens (id, player_id, token_hash, family_id, expires_at, is_revoked, created_at, used_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)";
    let params = vec![
        Value::from(token_id.to_string()),
        Value::from(player_id.to_string()),
        Value::from(token_hash.clone()),
        Value::from(family_id.to_string()),
        Value::from(expires_at.to_rfc3339()),
        Value::from(false),
        Value::from(now.to_rfc3339()),
        Value::from(None::<String>), // used_at is NULL
    ];
    
    let stmt = Statement::from_sql_and_values(DbBackend::Sqlite, sql, params);
    
    db.execute(stmt).await.map_err(|e| {
        eprintln!("Refresh token insert failed: {:?}", e);
        ApiError::DatabaseError(e)
    })?;
    
    Ok(TokenRotationResult {
        new_token: token,
        token_id,
        expires_at,
        player_id,
    })
}

/// Checks if a token is valid, reused, expired, or revoked.
pub async fn verify_refresh_token(token: &str) -> TokenVerificationResult {
    let db = get_db().await;
    let token_hash = hash_token(token);
    
    // Use raw SQL to avoid UUID decoding issues
    use sea_orm::{Statement, ConnectionTrait, Value, DbBackend};
    
    let sql = "SELECT id, player_id, family_id, expires_at, is_revoked, used_at FROM refresh_tokens WHERE token_hash = ?";
    let stmt = Statement::from_sql_and_values(DbBackend::Sqlite, sql, vec![Value::from(token_hash)]);
    
    match db.query_one(stmt).await {
        Ok(Some(row)) => {
            // Parse fields manually
            let id_str: String = match row.try_get("", "id") {
                Ok(s) => s,
                Err(_) => return TokenVerificationResult::NotFound,
            };
            let id = match Uuid::parse_str(&id_str) {
                Ok(u) => u,
                Err(_) => return TokenVerificationResult::NotFound,
            };
            let player_id_str: String = row.try_get("", "player_id").unwrap_or_default();
            let player_id = Uuid::parse_str(&player_id_str).unwrap_or(Uuid::nil());
            let family_id_str: String = row.try_get("", "family_id").unwrap_or_default();
            let family_id = Uuid::parse_str(&family_id_str).unwrap_or(Uuid::nil());
            let is_revoked: bool = row.try_get("", "is_revoked").unwrap_or(false);
            let used_at: Option<String> = row.try_get("", "used_at").ok();
            let expires_at_str: String = row.try_get("", "expires_at").unwrap_or_default();
            
            // Check if token is revoked
            if is_revoked {
                return TokenVerificationResult::Revoked;
            }
            
            // Check if token is expired
            if let Ok(expires_at) = chrono::DateTime::parse_from_rfc3339(&expires_at_str) {
                if expires_at.with_timezone(&Utc) < Utc::now() {
                    return TokenVerificationResult::Expired;
                }
            }
            
            // Check if token has been used before (THEFT DETECTION)
            if used_at.is_some() && !used_at.as_ref().unwrap().is_empty() {
                return TokenVerificationResult::Reused { family_id, player_id };
            }
            
            TokenVerificationResult::Valid { player_id, family_id, token_id: id }
        }
        Ok(None) => TokenVerificationResult::NotFound,
        Err(_) => TokenVerificationResult::NotFound,
    }
}

/// Marks a token as used - enables theft detection on reuse.
pub async fn mark_token_used(token_id: Uuid) -> Result<(), ApiError> {
    let db = get_db().await;
    use sea_orm::{Statement, ConnectionTrait, Value, DbBackend};
    
    // Atomic update: only update if used_at is NULL
    let sql = "UPDATE refresh_tokens SET used_at = CURRENT_TIMESTAMP WHERE id = ? AND used_at IS NULL";
    let stmt = Statement::from_sql_and_values(DbBackend::Sqlite, sql, vec![Value::from(token_id.to_string())]);
    
    let result = db.execute(stmt).await.map_err(ApiError::DatabaseError)?;
    
    if result.rows_affected() == 0 {
        // Update failed - either token doesn't exist OR it was already used
        // Check which case it is
        let check_sql = "SELECT used_at FROM refresh_tokens WHERE id = ?";
        let check_stmt = Statement::from_sql_and_values(DbBackend::Sqlite, check_sql, vec![Value::from(token_id.to_string())]);
        
        match db.query_one(check_stmt).await {
            Ok(Some(row)) => {
                let used_at: Option<String> = row.try_get("", "used_at").ok();
                if used_at.is_some() && !used_at.as_ref().unwrap().is_empty() {
                    // TOKEN REUSE DETECTED! Get family_id and invalidate
                    let family_sql = "SELECT family_id FROM refresh_tokens WHERE id = ?";
                    let family_stmt = Statement::from_sql_and_values(DbBackend::Sqlite, family_sql, vec![Value::from(token_id.to_string())]);
                    if let Ok(Some(frow)) = db.query_one(family_stmt).await {
                        let family_id_str: String = frow.try_get("", "family_id").unwrap_or_default();
                        if let Ok(family_id) = Uuid::parse_str(&family_id_str) {
                            let _ = invalidate_family(family_id).await;
                        }
                    }
                    return Err(ApiError::TokenTheftDetected);
                }
            }
            _ => {}
        }
        return Err(ApiError::InvalidToken);
    }
    
    Ok(())
}

/// Nukes an entire token family - called when theft is detected.
pub async fn invalidate_family(family_id: Uuid) -> Result<u64, ApiError> {
    let db = get_db().await;
    use sea_orm::{Statement, ConnectionTrait, Value, DbBackend};
    
    let sql = "UPDATE refresh_tokens SET is_revoked = 1 WHERE family_id = ? AND is_revoked = 0";
    let stmt = Statement::from_sql_and_values(DbBackend::Sqlite, sql, vec![Value::from(family_id.to_string())]);
    
    let result = db.execute(stmt).await.map_err(ApiError::DatabaseError)?;
    Ok(result.rows_affected())
}

/// The main rotation logic: verify old token, mint new one, mark old as used.
pub async fn rotate_token(old_token: &str) -> Result<TokenRotationResult, ApiError> {
    let config = TokenConfig::from_env();
    
    match verify_refresh_token(old_token).await {
        TokenVerificationResult::Valid { player_id, family_id, token_id } => {
            // Mark old token as used
            mark_token_used(token_id).await?;
            
            // Create new token in the same family
            create_refresh_token_with_family(player_id, family_id, &config).await
        }
        TokenVerificationResult::Reused { family_id, player_id: _ } => {
            // THEFT DETECTED! Invalidate entire family
            let _ = invalidate_family(family_id).await;
            Err(ApiError::TokenTheftDetected)
        }
        TokenVerificationResult::Expired => {
            Err(ApiError::TokenExpired)
        }
        TokenVerificationResult::NotFound => {
            Err(ApiError::InvalidToken)
        }
        TokenVerificationResult::Revoked => {
            Err(ApiError::TokenRevoked)
        }
    }
}

/// Logs out from all devices by revoking every token for a player.
pub async fn revoke_all_player_tokens(player_id: Uuid) -> Result<u64, ApiError> {
    let db = get_db().await;
    
    let tokens = RefreshToken::find()
        .filter(refresh_token::Column::PlayerId.eq(player_id))
        .filter(refresh_token::Column::IsRevoked.eq(false))
        .all(&db)
        .await
        .map_err(ApiError::DatabaseError)?;
    
    let mut count = 0u64;
    for token_record in tokens {
        let mut active_model: ActiveModel = token_record.into();
        active_model.is_revoked = Set(true);
        active_model.update(&db).await.map_err(ApiError::DatabaseError)?;
        count += 1;
    }
    
    Ok(count)
}
