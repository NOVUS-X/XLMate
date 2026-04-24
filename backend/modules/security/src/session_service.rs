use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};
use sea_orm::sea_query::Expr;
use uuid::Uuid;
use db_entity::session;
use std::fmt;

/// Errors that can occur during session operations
#[derive(Debug)]
pub enum SessionError {
    SessionNotFound,
    SessionExpired,
    SessionRevoked,
    DatabaseError(String),
}

impl fmt::Display for SessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SessionNotFound => write!(f, "Session not found"),
            Self::SessionExpired => write!(f, "Session has expired"),
            Self::SessionRevoked => write!(f, "Session has been revoked"),
            Self::DatabaseError(e) => write!(f, "Database error: {}", e),
        }
    }
}

impl From<DbErr> for SessionError {
    fn from(err: DbErr) -> Self {
        Self::DatabaseError(err.to_string())
    }
}

impl std::error::Error for SessionError {}

/// Session service for managing user sessions
#[derive(Clone, Debug)]
pub struct SessionService;

impl SessionService {
    /// Create a new session for a user
    pub async fn create_session(
        db: &DatabaseConnection,
        user_id: i32,
        ip_address: Option<String>,
        user_agent: Option<String>,
        ttl_hours: i64,
    ) -> Result<Uuid, SessionError> {
        let session_id = Uuid::new_v4();
        let now = Utc::now();
        let expires_at = now + chrono::Duration::hours(ttl_hours);

        let session_record = session::ActiveModel {
            id: Set(session_id),
            user_id: Set(user_id),
            ip_address: Set(ip_address),
            user_agent: Set(user_agent),
            created_at: Set(now),
            expires_at: Set(expires_at),
            is_revoked: Set(false),
            last_active_at: Set(now),
        };

        session_record.insert(db).await?;

        Ok(session_id)
    }

    /// Validate a session and update last active timestamp
    pub async fn validate_session(
        db: &DatabaseConnection,
        session_id: Uuid,
    ) -> Result<session::Model, SessionError> {
        let session_record = session::Entity::find()
            .filter(session::Column::Id.eq(session_id))
            .one(db)
            .await?;

        let session_record = session_record.ok_or(SessionError::SessionNotFound)?;

        // Check if revoked
        if session_record.is_revoked {
            return Err(SessionError::SessionRevoked);
        }

        // Check if expired
        if session_record.expires_at < Utc::now() {
            return Err(SessionError::SessionExpired);
        }

        // Update last active timestamp
        session::Entity::update_many()
            .col_expr(session::Column::LastActiveAt, Expr::value(Utc::now()))
            .filter(session::Column::Id.eq(session_id))
            .exec(db)
            .await?;

        Ok(session_record)
    }

    /// Revoke a specific session
    pub async fn revoke_session(
        db: &DatabaseConnection,
        session_id: Uuid,
    ) -> Result<(), SessionError> {
        session::Entity::update_many()
            .col_expr(session::Column::IsRevoked, Expr::value(true))
            .filter(session::Column::Id.eq(session_id))
            .exec(db)
            .await?;

        Ok(())
    }

    /// Revoke all sessions for a user (used for logout from all devices)
    pub async fn revoke_all_user_sessions(
        db: &DatabaseConnection,
        user_id: i32,
    ) -> Result<(), SessionError> {
        session::Entity::update_many()
            .col_expr(session::Column::IsRevoked, Expr::value(true))
            .filter(session::Column::UserId.eq(user_id))
            .exec(db)
            .await?;

        Ok(())
    }

    /// Get all active sessions for a user
    pub async fn get_user_sessions(
        db: &DatabaseConnection,
        user_id: i32,
    ) -> Result<Vec<session::Model>, SessionError> {
        let sessions = session::Entity::find()
            .filter(session::Column::UserId.eq(user_id))
            .filter(session::Column::IsRevoked.eq(false))
            .filter(session::Column::ExpiresAt.gt(Utc::now()))
            .all(db)
            .await?;

        Ok(sessions)
    }

    /// Clean up expired sessions (should be called periodically)
    pub async fn cleanup_expired_sessions(
        db: &DatabaseConnection,
    ) -> Result<u64, SessionError> {
        let result = session::Entity::delete_many()
            .filter(session::Column::ExpiresAt.lt(Utc::now()))
            .exec(db)
            .await?;

        Ok(result.rows_affected)
    }
}
