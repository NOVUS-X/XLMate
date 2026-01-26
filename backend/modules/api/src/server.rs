//! XLMate API Server Configuration
//!
//! This module sets up and configures the Actix-web server with all routes,
//! middleware, database connections, and authentication.

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;
use dotenv::dotenv;
use sea_orm::Database;
use std::env;

use security::{JwtService};

// Import route handlers
use crate::auth::{login, register};

/// Health check endpoint
async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
}

/// Welcome endpoint
async fn greet() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"message": "Welcome to XLMate API"}))
}

/// Main server initialization function
pub async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file
    dotenv().ok();

    // Initialize logger
    env_logger::init();

    // Load configuration from environment
    let server_addr = env::var("SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let jwt_secret = env::var("JWT_SECRET_KEY")
        .unwrap_or_else(|_| "xlmate_dev_secret_key_change_in_production".to_string());
    let jwt_expiration = env::var("JWT_EXPIRATION_SECS")
        .unwrap_or_else(|_| "3600".to_string())
        .parse::<usize>()
        .unwrap_or(3600);

    eprintln!("Initializing XLMate Backend Server");
    eprintln!("Server address: {}", server_addr);

    // Connect to database
    let db = match Database::connect(&database_url).await {
        Ok(conn) => {
            eprintln!("Database connection successful");
            conn
        }
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Database connection failed",
            ));
        }
    };

    // Initialize JWT service
    let jwt_service = JwtService::new(jwt_secret.clone(), jwt_expiration);

    eprintln!("Starting HTTP server on {}", server_addr);

    // Start HTTP server
    HttpServer::new(move || {
        let db = db.clone();
        let jwt_service = jwt_service.clone();
        let jwt_secret = jwt_secret.clone();

        // Configure CORS
        let cors = {
            let mut cors = Cors::default()
                .allow_any_method()
                .allow_any_header()
                .max_age(3600);

            if let Ok(allowed_origins) = env::var("ALLOWED_ORIGINS") {
                let origins: Vec<&str> = allowed_origins.split(',').collect();
                for origin in origins {
                    cors = cors.allowed_origin(origin.trim());
                }
                eprintln!("CORS configured with specific origins");
            } else {
                cors = cors.allow_any_origin();
                eprintln!("CORS configured to allow any origin (development mode)");
            }

            cors
        };

        App::new()
            // Global middleware
            .wrap(cors)
            // App data
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(jwt_service.clone()))
            // Health check
            .route("/health", web::get().to(health))
            .route("/", web::get().to(greet))
            // Authentication routes (public)
            .service(
                web::scope("/v1/auth")
                    .service(register)
                    .service(login),
            )
    })
    .bind(&server_addr)?
    .run()
    .await
}

