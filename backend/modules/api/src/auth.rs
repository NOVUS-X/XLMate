use actix_web::{
    HttpResponse, HttpRequest, post, cookie::{Cookie, SameSite},
    web::Json,
};
use chrono::{Duration, Utc};
use dto::{
    auth::{LoginRequest, LoginResponse, RegisterRequest, RefreshTokenRequest, TokenResponse},
    responses::{InvalidCredentialsResponse, ValidationErrorResponse},
};
use error::error::ApiError;
use jsonwebtoken::{encode, EncodingKey, Header};
use security::{Claims, create_refresh_token, rotate_token, revoke_all_player_tokens, TokenConfig, verify_refresh_token, TokenVerificationResult};
use serde_json::json;
use service::players::{add_player as create_player, get_player_by_username};
use service::helper::password::verify_password;
use validator::Validate;
use uuid::Uuid;

/// Generate a JWT access token for a player
fn generate_access_token(player_id: Uuid, config: &TokenConfig) -> Result<(String, i64), ApiError> {
    let now = Utc::now();
    let expires_in = 3600i64; // 1 hour
    let exp = (now + Duration::seconds(expires_in)).timestamp() as usize;
    let iat = now.timestamp() as usize;
    
    let claims = Claims {
        sub: player_id.to_string(),
        exp,
        iat,
    };
    
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    ).map_err(|_| ApiError::InvalidCredentials)?;
    
    Ok((token, expires_in))
}

/// Create a secure HTTP-only cookie for the refresh token
fn create_refresh_cookie(token: &str, ttl_days: i64) -> Cookie<'static> {
    Cookie::build("refresh_token", token.to_owned())
        .http_only(true)
        .secure(true) // Only sent over HTTPS
        .same_site(SameSite::Strict)
        .path("/v1/auth")
        .max_age(time::Duration::days(ttl_days))
        .finish()
}

/// Create a cookie to clear the refresh token
fn clear_refresh_cookie() -> Cookie<'static> {
    Cookie::build("refresh_token", "")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .path("/v1/auth")
        .max_age(time::Duration::seconds(0))
        .finish()
}

#[utoipa::path(
    post,
    path = "/v1/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials", body = InvalidCredentialsResponse)
    ),
    tag = "Authentication"
)]
#[post("/login")]
pub async fn login(payload: Json<LoginRequest>) -> HttpResponse {
    match payload.0.validate() {
        Ok(_) => {
            // Get player by username
            let player = match get_player_by_username(payload.0.username.clone()).await {
                Ok(Some(p)) => p,
                Ok(None) => return ApiError::InvalidCredentials.error_response(),
                Err(e) => return e.error_response(),
            };
            
            // Verify password
            let password_bytes = player.password_hash.clone();
            let stored_hash = String::from_utf8(password_bytes)
                .unwrap_or_default();
            
            if verify_password(&payload.0.password, &stored_hash).is_err() {
                return ApiError::InvalidCredentials.error_response();
            }
            
            let config = TokenConfig::from_env();
            
            // Generate access token
            let (access_token, expires_in) = match generate_access_token(player.id, &config) {
                Ok(t) => t,
                Err(e) => return e.error_response(),
            };
            
            // Create refresh token with new family
            let refresh_result = match create_refresh_token(player.id).await {
                Ok(r) => r,
                Err(e) => return e.error_response(),
            };
            
            // Build response with refresh token in cookie
            HttpResponse::Ok()
                .cookie(create_refresh_cookie(&refresh_result.new_token, config.refresh_token_ttl_days))
                .json(json!({
                    "access_token": access_token,
                    "token_type": "Bearer",
                    "expires_in": expires_in,
                    "user": {
                        "id": player.id,
                        "username": player.username,
                        "email": player.email
                    }
                }))
        }
        Err(errors) => ApiError::ValidationError(errors).error_response(),
    }
}

#[utoipa::path(
    post,
    path = "/v1/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "Registration successful", body = LoginResponse),
        (status = 400, description = "Invalid request parameters", body = InvalidCredentialsResponse),
        (status = 409, description = "User already exists", body = InvalidCredentialsResponse)
    ),
    tag = "Authentication"
)]
#[post("/register")]
pub async fn register(payload: Json<RegisterRequest>) -> HttpResponse {
    match payload.0.validate() {
        Ok(_) => {
            // Create the player
            let new_player_dto = dto::players::NewPlayer {
                username: payload.0.username.clone(),
                email: payload.0.email.clone(),
                password: payload.0.password.clone(),
                real_name: "".to_string(), // Default empty
            };
            
            let player = match create_player(new_player_dto).await {
                Ok(p) => p,
                Err(e) => return e.error_response(),
            };
            
            let config = TokenConfig::from_env();
            
            // Generate access token
            let (access_token, expires_in) = match generate_access_token(player.id, &config) {
                Ok(t) => t,
                Err(e) => return e.error_response(),
            };
            
            // Create refresh token with new family
            let refresh_result = match create_refresh_token(player.id).await {
                Ok(r) => r,
                Err(e) => return e.error_response(),
            };
            
            // Build response with refresh token in cookie
            HttpResponse::Created()
                .cookie(create_refresh_cookie(&refresh_result.new_token, config.refresh_token_ttl_days))
                .json(json!({
                    "access_token": access_token,
                    "token_type": "Bearer",
                    "expires_in": expires_in,
                    "user": {
                        "id": player.id,
                        "username": player.username,
                        "email": player.email
                    }
                }))
        }
        Err(errors) => ApiError::ValidationError(errors).error_response(),
    }
}

#[utoipa::path(
    post,
    path = "/v1/auth/refresh",
    request_body = Option<RefreshTokenRequest>,
    responses(
        (status = 200, description = "Token refreshed successfully", body = TokenResponse),
        (status = 401, description = "Invalid refresh token", body = InvalidCredentialsResponse),
        (status = 400, description = "Validation error", body = ValidationErrorResponse)
    ),
    tag = "Authentication"
)]
#[post("/refresh")]
pub async fn refresh_token(req: HttpRequest, payload: Option<Json<RefreshTokenRequest>>) -> HttpResponse {
    // Try to get token from cookie first, then from request body
    let token = req
        .cookie("refresh_token")
        .map(|c| c.value().to_owned())
        .or_else(|| payload.map(|p| p.0.refresh_token.clone()))
        .unwrap_or_default();
    
    if token.is_empty() {
        return ApiError::InvalidToken.error_response();
    }
    
    // Rotate the token
    match rotate_token(&token).await {
        Ok(rotation_result) => {
            let config = TokenConfig::from_env();
            
            // I use the player_id directly from the rotation result - no need to re-verify!
            let player_id = rotation_result.player_id;
            
            // Generate new access token
            let (access_token, expires_in) = match generate_access_token(player_id, &config) {
                Ok(t) => t,
                Err(e) => return e.error_response(),
            };
            
            // Return new tokens
            HttpResponse::Ok()
                .cookie(create_refresh_cookie(&rotation_result.new_token, config.refresh_token_ttl_days))
                .json(json!({
                    "access_token": access_token,
                    "token_type": "Bearer",
                    "expires_in": expires_in
                }))
        }
        Err(e) => {
            // Clear the cookie on error
            HttpResponse::Unauthorized()
                .cookie(clear_refresh_cookie())
                .json(json!({
                    "error": e.to_string(),
                    "code": 401,
                    "theft_detected": matches!(e, ApiError::TokenTheftDetected)
                }))
        }
    }
}

#[utoipa::path(
    post,
    path = "/v1/auth/logout",
    responses(
        (status = 200, description = "Logout successful"),
        (status = 401, description = "Unauthorized", body = InvalidCredentialsResponse)
    ),
    security(
        ("jwt_auth" = [])
    ),
    tag = "Authentication"
)]
#[post("/logout")]
pub async fn logout(req: HttpRequest) -> HttpResponse {
    // Get refresh token from cookie
    if let Some(cookie) = req.cookie("refresh_token") {
        let token = cookie.value();
        
        // Verify the token to get the player_id
        match verify_refresh_token(token).await {
            TokenVerificationResult::Valid { player_id, .. } => {
                // Revoke all tokens for this player (logout from all devices)
                let _ = revoke_all_player_tokens(player_id).await;
            }
            _ => {}
        }
    }
    
    // Clear the cookie
    HttpResponse::Ok()
        .cookie(clear_refresh_cookie())
        .json(json!({
            "message": "Logout successful"
        }))
}
