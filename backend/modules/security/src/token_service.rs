//! Refresh token rotation lives here, using "Token Families" for theft detection.
//! Reusing a token? The whole family gets invalidated - that's how we catch thieves.

use chrono::{Duration, Utc};
use db::db::db::get_db;
use entity::refresh_token::{self, ActiveModel, Entity as RefreshToken};
use error::error::ApiError;
use rand::Rng;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
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
            .unwrap_or_else(|_| "default-secret-change-me".to_string());
            
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
    
    let new_token = ActiveModel {
        id: Set(token_id),
        player_id: Set(player_id),
        token_hash: Set(token_hash),
        family_id: Set(family_id),
        expires_at: Set(expires_at.into()),
        is_revoked: Set(false),
        created_at: Set(now.into()),
        used_at: Set(None),
    };
    
    new_token.insert(&db).await.map_err(ApiError::DatabaseError)?;
    
    Ok(TokenRotationResult {
        new_token: token,
        token_id,
        expires_at,
    })
}

/// Checks if a token is valid, reused, expired, or revoked.
pub async fn verify_refresh_token(token: &str) -> TokenVerificationResult {
    let db = match get_db().await {
        db => db,
    };
    
    let token_hash = hash_token(token);
    
    let stored_token = RefreshToken::find()
        .filter(refresh_token::Column::TokenHash.eq(&token_hash))
        .one(&db)
        .await;
    
    match stored_token {
        Ok(Some(token_record)) => {
            // Check if token is revoked
            if token_record.is_revoked {
                return TokenVerificationResult::Revoked;
            }
            
            // Check if token is expired
            let now = Utc::now();
            if token_record.expires_at.with_timezone(&Utc) < now {
                return TokenVerificationResult::Expired;
            }
            
            // Check if token has been used before (THEFT DETECTION)
            if token_record.used_at.is_some() {
                return TokenVerificationResult::Reused {
                    family_id: token_record.family_id,
                    player_id: token_record.player_id,
                };
            }
            
            TokenVerificationResult::Valid {
                player_id: token_record.player_id,
                family_id: token_record.family_id,
                token_id: token_record.id,
            }
        }
        Ok(None) => TokenVerificationResult::NotFound,
        Err(_) => TokenVerificationResult::NotFound,
    }
}

/// Marks a token as used - enables theft detection on reuse.
pub async fn mark_token_used(token_id: Uuid) -> Result<(), ApiError> {
    let db = get_db().await;
    
    let token = RefreshToken::find_by_id(token_id)
        .one(&db)
        .await
        .map_err(ApiError::DatabaseError)?;
    
    if let Some(token_record) = token {
        let mut active_model: ActiveModel = token_record.into();
        active_model.used_at = Set(Some(Utc::now().into()));
        active_model.update(&db).await.map_err(ApiError::DatabaseError)?;
    }
    
    Ok(())
}

/// Nukes an entire token family - called when theft is detected.
pub async fn invalidate_family(family_id: Uuid) -> Result<u64, ApiError> {
    let db = get_db().await;
    
    let tokens = RefreshToken::find()
        .filter(refresh_token::Column::FamilyId.eq(family_id))
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
