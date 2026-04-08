#[cfg(test)]
pub mod test_service {
    use crate::helper::password;
    use db::test_helpers::setup_test_db;
    use dto::players::{NewPlayer, UpdatePlayer};
    use db_entity::player::{self, Model, ActiveModelTrait, Set};
    use error::error::ApiError;
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    use uuid::Uuid;

    pub async fn add_test_player(new_player: NewPlayer) -> Result<Model, ApiError> {
        let db = setup_test_db().await;

        // Check if username or email already exists
        if is_username_taken_test(&db, &new_player.username).await {
            return Err(ApiError::BadRequest("Username already exists".to_string()));
        }

        if is_email_taken_test(&db, &new_player.email).await {
            return Err(ApiError::BadRequest("Email already exists".to_string()));
        }

        // Remove .await - hash_password is synchronous
        let hashed_password = password::hash_password(&new_player.password)
            .map_err(|e| ApiError::PasswordHashError(e))?;

        let player = player::ActiveModel {
            id: Set(Uuid::new_v4()),
            username: Set(new_player.username),
            email: Set(new_player.email),
            password_hash: Set(hashed_password.as_bytes().to_vec()),
            real_name: Set(new_player.real_name),
            elo_rating: Set(1200),
            is_enabled: Set(true),
            ..Default::default()
        };

        let result = player.insert(&db).await
            .map_err(|e| ApiError::DatabaseError(e))?;

        Ok(result)
    }

    async fn is_username_taken_test(db: &sea_orm::DatabaseConnection, username: &str) -> bool {
        let user = player::Entity::find()
            .filter(player::Column::Username.eq(username))
            .one(db)
            .await
            .unwrap_or_default();

        user.is_some()
    }

    async fn is_email_taken_test(db: &sea_orm::DatabaseConnection, email: &str) -> bool {
        match player::Entity::find()
            .filter(player::Column::Email.eq(email))
            .one(db)
            .await
        {
            Ok(user) => user.is_some(),
            Err(_) => false,
        }
    }
}
