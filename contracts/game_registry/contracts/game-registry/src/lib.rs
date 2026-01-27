#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameResult {
    pub winner: Address,
    pub white: Address,
    pub black: Address,
    pub timestamp: u64,
}

#[contracttype]
pub enum DataKey {
    Admin,
    Server,
    Game(String),
}

#[contract]
pub struct GameRegistry;

#[contractimpl]
impl GameRegistry {
    /// Initialize the contract with an admin and an authorized server address.
    pub fn initialize(env: Env, admin: Address, server: Address) {
        if env.storage().persistent().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&DataKey::Server, &server);

        // Extend TTL for Admin and Server keys to prevent expiration
        env.storage().persistent().extend_ttl(&DataKey::Admin, 100_000, 500_000);
        env.storage().persistent().extend_ttl(&DataKey::Server, 100_000, 500_000);
    }

    /// Records a game result. Only the authorized server can call this.
    pub fn record_game(
        env: Env,
        game_id: String,
        winner: Address,
        white: Address,
        black: Address,
        timestamp: u64,
    ) {
        let server: Address = env.storage().persistent().get(&DataKey::Server).expect("Not initialized");
        server.require_auth();

        if env.storage().persistent().has(&DataKey::Game(game_id.clone())) {
            panic!("Game already recorded");
        }

        let result = GameResult {
            winner: winner.clone(),
            white,
            black,
            timestamp,
        };

        let key = DataKey::Game(game_id.clone());
        env.storage().persistent().set(&key, &result);
        
        // Extend TTL for the game record to ensure it stays active.
        // We use some reasonable defaults for persistent storage.
        env.storage().persistent().extend_ttl(&key, 100_000, 500_000);

        // Emit GameFinalized event
        // (Topic, Data)
        env.events().publish(
            (Symbol::new(&env, "GameFinalized"), game_id),
            (winner, timestamp),
        );
    }

    /// Retrieves a recorded game result.
    pub fn get_game(env: Env, game_id: String) -> GameResult {
        env.storage()
            .persistent()
            .get(&DataKey::Game(game_id))
            .expect("Game not found")
    }

    /// Updates the authorized server address. Only the admin can call this.
    pub fn set_server(env: Env, new_server: Address) {
        let admin: Address = env.storage().persistent().get(&DataKey::Admin).expect("Not initialized");
        admin.require_auth();
        env.storage().persistent().set(&DataKey::Server, &new_server);
        env.storage().persistent().extend_ttl(&DataKey::Server, 100_000, 500_000);
    }

    /// Updates the admin address. Only the current admin can call this.
    pub fn set_admin(env: Env, new_admin: Address) {
        let admin: Address = env.storage().persistent().get(&DataKey::Admin).expect("Not initialized");
        admin.require_auth();
        env.storage().persistent().set(&DataKey::Admin, &new_admin);
        env.storage().persistent().extend_ttl(&DataKey::Admin, 100_000, 500_000);
    }
}

mod test;
