//! XLMate Backend Server Binary
//!
//! Entry point for running the XLMate API server.
//! Delegates to the server module for initialization.

use api::server;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    server::main().await
}
