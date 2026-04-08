#[cfg(test)]
use actix_web::{HttpResponse, web::Json};
use dto::players::{NewPlayer};
use error::error::ApiError;
use serde_json::json;
use validator::Validate;
use db_entity::player::Model;

pub async fn add_test_player_endpoint(payload: Json<NewPlayer>) -> HttpResponse {
    match payload.0.validate() {
        Ok(_) => {
            // For testing, we'll create a mock response without database
            let mock_player = Model {
                id: uuid::Uuid::new_v4(),
                username: payload.0.username.clone(),
                email: payload.0.email.clone(),
                password_hash: vec![],
                biography: "".to_string(),
                country: "".to_string(),
                flair: "".to_string(),
                real_name: payload.0.real_name.clone(),
                location: None,
                fide_rating: None,
                elo_rating: 1200,
                social_links: None,
                is_enabled: true,
            };

            HttpResponse::Ok().json(json!({
                "message":"New player added",
                "data": mock_player
            }))
        }
        Err(errors) => ApiError::ValidationError(errors).error_response(),
    }
}
