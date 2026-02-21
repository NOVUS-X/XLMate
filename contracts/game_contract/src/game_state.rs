#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameState {
    pub board: String, // FEN representation of the board
    pub current_turn: String, // "white" or "black"
    pub status: String, // "in_progress", "checkmate", "stalemate", etc.
}

#[contracttype]
pub enum DataKey {
    Game(String),
}

#[contract]
pub struct GameContract;

#[contractimpl]
impl GameContract {
    pub fn create_game(env: Env, game_id: String, initial_board: String) {
        let state = GameState {
            board: initial_board,
            current_turn: "white".into(),
            status: "in_progress".into(),
        };
        
        let key = DataKey::Game(game_id);
        env.storage().persistent().set(&key, &state);
    }

    pub fn get_game(env: Env, game_id: String) -> Option<GameState> {
        let key = DataKey::Game(game_id);
        env.storage().persistent().get(&key)
    }

    pub fn make_move(env: Env, game_id: String, new_board: String, next_turn: String) {
        let key = DataKey::Game(game_id);
        if let Some(mut state) = env.storage().persistent().get::<GameState>(&key) {
            state.board = new_board;
            state.current_turn = next_turn;
            env.storage().persistent().set(&key, &state);
        }
    }

    pub fn finalize_game(env: Env, game_id: String, final_status: String) {
        let key = DataKey::Game(game_id);
        if let Some(mut state) = env.storage().persistent().get::<GameState>(&key) {
            state.status = final_status;
            env.storage().persistent().set(&key, &state);
        }
    }
}