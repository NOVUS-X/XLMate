#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env, String};

#[test]
fn test_game_registry_success() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let server = Address::generate(&env);
    let player1 = Address::generate(&env);
    let player2 = Address::generate(&env);

    let contract_id = env.register(GameRegistry, ());
    let client = GameRegistryClient::new(&env, &contract_id);

    // Initialize
    client.initialize(&admin, &server);

    let game_id = String::from_str(&env, "game-123");
    let timestamp = 1737500000u64;
    
    // Record game by server
    client.record_game(&game_id, &player1, &player1, &player2, &timestamp);

    // Verify game retrieval
    let recorded_game = client.get_game(&game_id);
    assert_eq!(recorded_game.winner, player1);
    assert_eq!(recorded_game.white, player1);
    assert_eq!(recorded_game.black, player2);
    assert_eq!(recorded_game.timestamp, timestamp);
}

#[test]
#[should_panic]
fn test_unauthorized_record() {
    let env = Env::default();
    // We NOT calling env.mock_all_auths() here makes require_auth fail unless we manually setup auth.
    
    let admin = Address::generate(&env);
    let server = Address::generate(&env);
    let player = Address::generate(&env);

    let contract_id = env.register(GameRegistry, ());
    let client = GameRegistryClient::new(&env, &contract_id);

    client.initialize(&admin, &server);

    let game_id = String::from_str(&env, "fail");
    // This should panic because 'server' has not authorized the call.
    client.record_game(&game_id, &player, &player, &player, &1);
}

#[test]
fn test_update_server_and_admin() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let server = Address::generate(&env);
    let new_server = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let contract_id = env.register(GameRegistry, ());
    let client = GameRegistryClient::new(&env, &contract_id);

    client.initialize(&admin, &server);

    // Change server
    client.set_server(&new_server);
    
    // Change admin
    client.set_admin(&new_admin);

    // Verify we can still record with new server
    let game_id = String::from_str(&env, "game-456");
    client.record_game(&game_id, &new_admin, &new_admin, &new_admin, &456);
}

#[test]
#[should_panic(expected = "Already initialized")]
fn test_double_initialize() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let server = Address::generate(&env);

    let contract_id = env.register(GameRegistry, ());
    let client = GameRegistryClient::new(&env, &contract_id);

    client.initialize(&admin, &server);
    client.initialize(&admin, &server);
}
