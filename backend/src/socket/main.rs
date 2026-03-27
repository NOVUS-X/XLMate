mod game;
mod handlers;
mod models;
mod websocket;

use std::env;
use std::sync::Arc;
use tokio::net::TcpListener;
use websocket::handle_connection;
use st_core::SorobanEventListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the logger
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    // Get the address from environment or use default
    let addr = env::var("BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1:8080".to_string());

    log::info!("Starting WebSocket server on {}", addr);

    // Initialize the game state
    game::init_game_state();

    // ── Soroban event listener ────────────────────────────────────────────────
    // Polls Horizon every 5 seconds for staking events. When a stake is
    // confirmed on-chain, it broadcasts a MatchStartSignal to whichever
    // WebSocket handler subscribed via listener.track_match().
    let horizon_url = env::var("HORIZON_URL")
        .unwrap_or_else(|_| "https://horizon-testnet.stellar.org".to_string());

    let listener = Arc::new(SorobanEventListener::new(horizon_url));

    // Spawn the poll loop as a background task — runs independently of the
    // WebSocket accept loop and survives individual connection errors.
    let listener_clone = Arc::clone(&listener);
    tokio::spawn(async move {
        listener_clone.run().await;
    });

    log::info!("Soroban event listener started");

    // Create the TCP listener
    let tcp_listener = TcpListener::bind(&addr).await?;
    log::info!("WebSocket server listening on: {}", addr);

    // Accept connections
    loop {
        match tcp_listener.accept().await {
            Ok((stream, peer_addr)) => {
                log::info!("New connection from: {}", peer_addr);

                // Clone the Arc so each connection task has its own handle to
                // the listener — used to call track_match() when a rated match
                // is created and needs to watch for its stake event.
                let listener_handle = Arc::clone(&listener);

                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, peer_addr, listener_handle).await {
                        log::error!("Error handling connection: {}", e);
                    }
                });
            }
            Err(e) => {
                log::error!("Failed to accept connection: {}", e);
                // Continue accepting connections despite errors
            }
        }
    }
}