use actix_web::{web, HttpResponse, post};
use validator::Validate;

use dto::auth::{RegisterRequest, LoginRequest, AuthResponse, ErrorResponse};
use security::JwtService;
use sea_orm::DatabaseConnection;

/// Register a new user
#[post("/register")]
pub async fn register(
    _db: web::Data<DatabaseConnection>,
    payload: web::Json<RegisterRequest>,
) -> HttpResponse {
    // Validate input
    if let Err(errors) = payload.validate() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            message: format!("Validation failed: {:?}", errors),
            code: "VALIDATION_ERROR".to_string(),
        });
    }

    // For now, return a mock response
    HttpResponse::Created().json(AuthResponse {
        access_token: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...".to_string(),
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        user_id: 1,
        username: payload.username.clone(),
    })
}

/// Login with credentials
#[post("/login")]
pub async fn login(
    _db: web::Data<DatabaseConnection>,
    payload: web::Json<LoginRequest>,
    jwt_service: web::Data<JwtService>,
) -> HttpResponse {
    // Validate input
    if let Err(errors) = payload.validate() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            message: format!("Validation failed: {:?}", errors),
            code: "VALIDATION_ERROR".to_string(),
        });
    }

    // For now, return a mock response with generated token
    match jwt_service.generate_token(1, &payload.username) {
        Ok(token) => {
            HttpResponse::Ok().json(AuthResponse {
                access_token: token,
                token_type: "Bearer".to_string(),
                expires_in: 3600,
                user_id: 1,
                username: payload.username.clone(),
            })
        }
        Err(_) => {
            HttpResponse::InternalServerError().json(ErrorResponse {
                message: "Failed to generate token".to_string(),
                code: "TOKEN_ERROR".to_string(),
            })
        }
    }
}
