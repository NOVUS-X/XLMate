pub mod auth;
// pub mod players;
// pub mod games;
pub mod server;
// pub mod ai;
// pub mod openapi;
// pub mod ws;
// mod test;

// Re-export server module for external use
pub use server::main;
pub use auth::{login, register};
