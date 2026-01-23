use crate::helper::password;
use db::db::db::get_db;
use dto::players::{NewPlayer, UpdatePlayer};
use entity::player::{self, Model};
use error::error::ApiError;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

// Helper to check if username exists using Raw SQL
async fn is_username_taken(username: String) -> bool {
    let db = get_db().await;
    use sea_orm::{Statement, ConnectionTrait, Value, DbBackend};
    
    let sql = "SELECT count(*) FROM player WHERE username = ?";
    let stmt = Statement::from_sql_and_values(DbBackend::Sqlite, sql, vec![Value::from(username.clone())]);
    
    match db.query_one(stmt).await {
        Ok(Some(res)) => {
            let count: i64 = res.try_get("", "count(*)").unwrap_or(0);
            eprintln!("DEBUG: is_username_taken('{}') = {} (count={})", username, count > 0, count);
            count > 0
        },
        Ok(None) => {
            eprintln!("DEBUG: is_username_taken('{}') = false (no result)", username);
            false
        },
        Err(e) => {
            eprintln!("DEBUG: is_username_taken('{}') = false (error: {:?})", username, e);
            false
        }
    }
}

// Helper to check if email exists using Raw SQL
async fn is_email_taken(email: String) -> bool {
    let db = get_db().await;
    use sea_orm::{Statement, ConnectionTrait, Value, DbBackend};
    
    let sql = "SELECT count(*) FROM player WHERE email = ?";
    let stmt = Statement::from_sql_and_values(DbBackend::Sqlite, sql, vec![Value::from(email.clone())]);
    
    match db.query_one(stmt).await {
        Ok(Some(res)) => {
            let count: i64 = res.try_get("", "count(*)").unwrap_or(0);
            eprintln!("DEBUG: is_email_taken('{}') = {} (count={})", email, count > 0, count);
            count > 0
        },
        Ok(None) => {
            eprintln!("DEBUG: is_email_taken('{}') = false (no result)", email);
            false
        },
        Err(e) => {
            eprintln!("DEBUG: is_email_taken('{}') = false (error: {:?})", email, e);
            false
        }
    }
}

pub async fn find_player_by_id(id: Uuid) -> Result<player::Model, ApiError> {
    let db = get_db().await;
    use sea_orm::{Statement, ConnectionTrait, Value, DbBackend};
    
    // Raw SQL fetch with manual result parsing to bypass SeaORM UUID decoding issues
    let sql = "SELECT id, username, email, password_hash, biography, country, flair, real_name, location, fide_rating, social_links, is_enabled FROM player WHERE id = ? AND is_enabled = 1";
    let stmt = Statement::from_sql_and_values(DbBackend::Sqlite, sql, vec![Value::from(id.to_string())]);
    
    match db.query_one(stmt).await {
        Ok(Some(row)) => {
            // Manually extract values from QueryResult
            let id_str: String = row.try_get("", "id").map_err(|e| ApiError::DatabaseError(sea_orm::DbErr::Custom(e.to_string())))?;
            let id = Uuid::parse_str(&id_str).map_err(|e| ApiError::DatabaseError(sea_orm::DbErr::Custom(e.to_string())))?;
            
            Ok(player::Model {
                id,
                username: row.try_get("", "username").unwrap_or_default(),
                email: row.try_get("", "email").unwrap_or_default(),
                password_hash: row.try_get("", "password_hash").unwrap_or_default(),
                biography: row.try_get("", "biography").unwrap_or_default(),
                country: row.try_get("", "country").unwrap_or_default(),
                flair: row.try_get("", "flair").unwrap_or_default(),
                real_name: row.try_get("", "real_name").unwrap_or_default(),
                location: row.try_get("", "location").ok(),
                fide_rating: row.try_get("", "fide_rating").ok(),
                social_links: row.try_get("", "social_links").ok(),
                is_enabled: row.try_get("", "is_enabled").unwrap_or(true),
            })
        },
        Ok(None) => Err(ApiError::NotFound(format!("Player {}", id))),
        Err(e) => Err(ApiError::DatabaseError(e)),
    }
}

pub async fn get_player_by_username(username: String) -> Result<Option<Model>, ApiError> {
    let db = get_db().await;
    use sea_orm::{Statement, ConnectionTrait, Value, DbBackend};

    let sql = "SELECT id, username, email, password_hash, biography, country, flair, real_name, location, fide_rating, social_links, is_enabled FROM player WHERE username = ?";
    let stmt = Statement::from_sql_and_values(DbBackend::Sqlite, sql, vec![Value::from(username)]);
    
    match db.query_one(stmt).await {
        Ok(Some(row)) => {
            let id_str: String = row.try_get("", "id").map_err(|e| ApiError::DatabaseError(sea_orm::DbErr::Custom(e.to_string())))?;
            let id = Uuid::parse_str(&id_str).map_err(|e| ApiError::DatabaseError(sea_orm::DbErr::Custom(e.to_string())))?;
            
            Ok(Some(player::Model {
                id,
                username: row.try_get("", "username").unwrap_or_default(),
                email: row.try_get("", "email").unwrap_or_default(),
                password_hash: row.try_get("", "password_hash").unwrap_or_default(),
                biography: row.try_get("", "biography").unwrap_or_default(),
                country: row.try_get("", "country").unwrap_or_default(),
                flair: row.try_get("", "flair").unwrap_or_default(),
                real_name: row.try_get("", "real_name").unwrap_or_default(),
                location: row.try_get("", "location").ok(),
                fide_rating: row.try_get("", "fide_rating").ok(),
                social_links: row.try_get("", "social_links").ok(),
                is_enabled: row.try_get("", "is_enabled").unwrap_or(true),
            }))
        },
        Ok(None) => Ok(None),
        Err(e) => Err(ApiError::DatabaseError(e)),
    }
}

pub async fn add_player(payload: NewPlayer) -> Result<player::Model, ApiError> {
    let email_available = is_email_taken(payload.email.clone()).await;
    let username_available = is_username_taken(payload.username.clone()).await;
    // Check if EXISTS (taken means count > 0)
    // Variable names are 'available', so should be !taken.
    // Implementation of is_username_taken returns TRUE if count > 0 (TAKEN).
    // So 'username_available' variable name is misleading. Let's fix logic.
    if email_available || username_available { // If taken (true)
        return Err(ApiError::InvalidCredentials); // Or Conflict
    }
    
    let id = Uuid::new_v4();
    let hashed_password = password::hash_password(&payload.password)?.into_bytes();

    let db = get_db().await;
    
    // Workaround: Raw SQL insert to bypass "String unsupported" error with SeaORM/SQLx/File-SQLite
    // Explicitly bind UUID as string and handle other fields.
    use sea_orm::{Statement, ConnectionTrait, Value, DbBackend};
    
    // We must match the column order expected or name them.
    // Columns: id, username, email, password_hash, real_name, is_enabled (default), created_at (default), ...
    // Note: Migration m20260124 adds is_enabled (default true).
    
    let sql = "INSERT INTO player (id, username, email, password_hash, real_name) VALUES (?, ?, ?, ?, ?)";
    let params = vec![
        Value::from(id.to_string()), // Explicit String
        Value::from(payload.username),
        Value::from(payload.email),
        Value::from(hashed_password), // Bytes
        Value::from(payload.real_name),
    ];
    
    let stmt = Statement::from_sql_and_values(DbBackend::Sqlite, sql, params);
    
    match db.execute(stmt).await {
        Ok(_) => {
            find_player_by_id(id).await
        },
        Err(err) => {
             // Log error content for debugging
             eprintln!("Raw Insert Failed: {:?}", err);
             Err(ApiError::DatabaseError(err))
        },
    }
}

pub async fn update_player(id: Uuid, payload: UpdatePlayer) -> Result<player::Model, ApiError> {
    let db = get_db().await;
    let existing_player = find_player_by_id(id).await?;

    let mut active_model: player::ActiveModel = existing_player.clone().into();

    if let Some(biography) = payload.biography {
        active_model.biography = Set(biography);
    }
    if let Some(real_name) = payload.real_name {
        active_model.real_name = Set(real_name);
    }
    if let Some(country) = payload.country {
        active_model.country = Set(country);
    }
    if let Some(flair) = payload.flair {
        active_model.flair = Set(flair);
    }
    if let Some(location) = payload.location {
        active_model.location = Set(Some(location));
    }
    if let Some(fide_rating) = payload.fide_rating {
        active_model.fide_rating = Set(Some(fide_rating));
    }
    if let Some(social_links) = payload.social_links {
        active_model.social_links = Set(Some(social_links));
    }
    if let Some(ref username) = payload.username {
        let existing_username = get_player_by_username(username.clone()).await?;
        match existing_username {
            Some(ref user) => {
                if user.email == existing_player.email {
                    active_model.username = Set(username.clone());
                }
            }
            None => {
                active_model.username = Set(username.clone());
            }
        }
    }

    let updated_player = active_model
        .update(&db)
        .await
        .map_err(ApiError::DatabaseError)?;

    Ok(updated_player)
}

pub async fn delete_player(id: Uuid) -> Result<(), ApiError> {
    let db = get_db().await;
    let existing_player = find_player_by_id(id).await?;

    let mut active_model: player::ActiveModel = existing_player.into();

    active_model.is_enabled = Set(false);

    active_model
        .update(&db)
        .await
        .map_err(ApiError::DatabaseError)?;

    Ok(())
}
