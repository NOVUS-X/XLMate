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
#[should_panic]
fn test_double_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let server = Address::generate(&env);

    let contract_id = env.register(GameRegistry, ());
    let client = GameRegistryClient::new(&env, &contract_id);

    client.initialize(&admin, &server);
    client.initialize(&admin, &server); // Should panic with AlreadyInitialized error
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #5)")]
fn test_tournament_full() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let server = Address::generate(&env);
    let player1 = Address::generate(&env);
    let player2 = Address::generate(&env);
    let player3 = Address::generate(&env);

    let contract_id = env.register(GameRegistry, ());
    let client = GameRegistryClient::new(&env, &contract_id);

    client.initialize(&admin, &server);

    let token_address = Address::generate(&env);
    let tournament_id = String::from_str(&env, "tourney-1");

    client.create_tournament(&tournament_id, &2, &0, &token_address);
    client.register_tournament(&player1, &tournament_id);
    client.register_tournament(&player2, &tournament_id);

    // This should panic with TournamentFull (5)
    client.register_tournament(&player3, &tournament_id);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6)")]
fn test_already_registered() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let server = Address::generate(&env);
    let player1 = Address::generate(&env);

    let contract_id = env.register(GameRegistry, ());
    let client = GameRegistryClient::new(&env, &contract_id);

    client.initialize(&admin, &server);

    let token_address = Address::generate(&env);
    let tournament_id = String::from_str(&env, "tourney-2");

    client.create_tournament(&tournament_id, &2, &0, &token_address);
    client.register_tournament(&player1, &tournament_id);
    
    // This should panic with AlreadyRegistered (6)
    client.register_tournament(&player1, &tournament_id);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7)")]
fn test_insufficient_entry_fee() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let server = Address::generate(&env);
    let player1 = Address::generate(&env);

    let contract_id = env.register(GameRegistry, ());
    let client = GameRegistryClient::new(&env, &contract_id);

    client.initialize(&admin, &server);

    // Register a mock token contract
    // For testing insufficient balance, we can just use a non-existent token contract 
    // or one where player has 0 balance. Let's just use a generated address.
    // The token client will fail to get balance, or return 0, leading to InsufficientEntryFee.
    // Actually, calling a non-existent contract might panic with a different error (HostError: Error(WasmVm, ...))
    // So we should register a real token contract for the test to reach the balance check properly,
    // or rely on the mock auth which might just mock the token client call.
    // Wait, env.mock_all_auths() mocks the `require_auth` calls, but cross-contract calls 
    // require the contract to exist.
    // Let's just use the built-in token contract.
    let token_admin = Address::generate(&env);
    let token_contract_id = env.register_stellar_asset_contract_v2(token_admin);
    let token_address = token_contract_id.address();

    let tournament_id = String::from_str(&env, "tourney-3");

    // Entry fee is 100
    client.create_tournament(&tournament_id, &2, &100, &token_address);
    
    // Player 1 has 0 balance, so this should panic with InsufficientEntryFee (7)
    client.register_tournament(&player1, &tournament_id);
}
