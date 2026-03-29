#![cfg(test)]
extern crate std;

use super::*;
use soroban_sdk::token::{StellarAssetClient, TokenClient};
use soroban_sdk::{Address, Bytes, Env, Map, Vec, testutils::Address as _};

/// Helper: seed a completed game directly into contract storage, bypassing
/// token transfers and auth checks.  Returns the game_id (always 1).
fn seed_completed_game(
    env: &Env,
    contract_id: &Address,
    player1: &Address,
    player2: &Address,
    wager: i128,
) -> u64 {
    let game_id: u64 = 1;
    env.as_contract(contract_id, || {
        // Write game counter
        env.storage().instance().set(&GAME_COUNTER, &game_id);

        // Build a completed game
        let game = Game {
            id: game_id,
            player1: player1.clone(),
            player2: Some(player2.clone()),
            state: GameState::Completed,
            wager_amount: wager,
            current_turn: 1,
            moves: Vec::new(env),
            created_at: 0,
            winner: None,
        };
        let mut games: Map<u64, Game> = Map::new(env);
        games.set(game_id, game);
        env.storage().instance().set(&GAMES, &games);

        // Seed escrow so payout_tournament can debit both players
        let mut escrow: Map<Address, i128> = Map::new(env);
        escrow.set(player1.clone(), wager);
        escrow.set(player2.clone(), wager);
        env.storage().instance().set(&ESCROW, &escrow);
    });
    game_id
}

#[test]
fn test_payout_tournament() {
    let env = Env::default();
    let contract_id = env.register_contract(None, GameContract);
    let client = GameContractClient::new(&env, &contract_id);

    let player1 = Address::generate(&env);
    let player2 = Address::generate(&env);
    let wager: i128 = 1000;

    let game_id = seed_completed_game(&env, &contract_id, &player1, &player2, wager);

    let winner1 = Address::generate(&env);
    let winner2 = Address::generate(&env);
    let winner3 = Address::generate(&env);

    let mut winners = Vec::new(&env);
    winners.push_back(winner1.clone());
    winners.push_back(winner2.clone());
    winners.push_back(winner3.clone());

    let mut percentages = Vec::new(&env);
    percentages.push_back(50);
    percentages.push_back(30);
    percentages.push_back(20);

    // Call payout_tournament
    client
        .mock_all_auths()
        .payout_tournament(&game_id, &winners, &percentages);

    // Total pool should be wager * 2 = 2000
    // Expected payouts: 50% = 1000, 30% = 600, 20% = 400
    env.as_contract(&contract_id, || {
        let escrow: Map<Address, i128> = env.storage().instance().get(&ESCROW).unwrap();

        // Assert sum precisely equals total pool
        let w1_escrow = escrow.get(winner1.clone()).unwrap_or(0);
        let w2_escrow = escrow.get(winner2.clone()).unwrap_or(0);
        let w3_escrow = escrow.get(winner3.clone()).unwrap_or(0);

        assert_eq!(w1_escrow, 1000);
        assert_eq!(w2_escrow, 600);
        assert_eq!(w3_escrow, 400);

        // Calculate total sum of payouts
        let total_distributed = w1_escrow + w2_escrow + w3_escrow;
        assert_eq!(total_distributed, (wager * 2) as i128);

        // Player1 and Player2 escrows should be subtracted by wager amount
        let p1_escrow = escrow.get(player1.clone()).unwrap_or(0);
        let p2_escrow = escrow.get(player2.clone()).unwrap_or(0);
        assert_eq!(p1_escrow, 0); // Started as 1000, subtracted 1000
        assert_eq!(p2_escrow, 0); // Started as 1000, subtracted 1000
    });
}

#[test]
fn test_payout_tournament_dust() {
    let env = Env::default();
    let contract_id = env.register_contract(None, GameContract);
    let client = GameContractClient::new(&env, &contract_id);

    let player1 = Address::generate(&env);
    let player2 = Address::generate(&env);

    // An amount that creates an uneven division for testing "precision" remainder distribution
    let wager: i128 = 333; // total pool = 666

    let game_id = seed_completed_game(&env, &contract_id, &player1, &player2, wager);

    let winner1 = Address::generate(&env);
    let winner2 = Address::generate(&env);
    let winner3 = Address::generate(&env);

    let mut winners = Vec::new(&env);
    winners.push_back(winner1.clone());
    winners.push_back(winner2.clone());
    winners.push_back(winner3.clone());

    let mut percentages = Vec::new(&env);
    percentages.push_back(50); // 333
    percentages.push_back(30); // 199.8 -> 199
    percentages.push_back(20); // 133.2 -> 133
    // Sum without remainder distribution: 333 + 199 + 133 = 665
    // Remainder: 666 - 665 = 1
    // With remainder to first place: w1 gets 333 + 1 = 334.

    client
        .mock_all_auths()
        .payout_tournament(&game_id, &winners, &percentages);

    env.as_contract(&contract_id, || {
        let escrow: Map<Address, i128> = env.storage().instance().get(&ESCROW).unwrap();

        let w1_escrow = escrow.get(winner1.clone()).unwrap_or(0);
        let w2_escrow = escrow.get(winner2.clone()).unwrap_or(0);
        let w3_escrow = escrow.get(winner3.clone()).unwrap_or(0);

        assert_eq!(w1_escrow, 334);
        assert_eq!(w2_escrow, 199);
        assert_eq!(w3_escrow, 133);

        let total_distributed = w1_escrow + w2_escrow + w3_escrow;
        assert_eq!(total_distributed, (wager * 2) as i128); // 666
    });
}

#[test]
fn test_payout_tournament_invalid_percentage() {
    let env = Env::default();
    let contract_id = env.register_contract(None, GameContract);
    let client = GameContractClient::new(&env, &contract_id);

    let player1 = Address::generate(&env);
    let player2 = Address::generate(&env);
    let wager: i128 = 1000;

    let game_id = seed_completed_game(&env, &contract_id, &player1, &player2, wager);

    let winner1 = Address::generate(&env);

    let mut winners = Vec::new(&env);
    winners.push_back(winner1.clone());

    let mut percentages = Vec::new(&env);
    percentages.push_back(90); // Does not equal 100

    let res = client
        .mock_all_auths()
        .try_payout_tournament(&game_id, &winners, &percentages);

    // Result should be Err matching InvalidPercentage (12)
    assert!(res.is_err());
}

#[test]
fn test_create_game_exceeds_max_stake() {
    let env = Env::default();
    let contract_id = env.register_contract(None, GameContract);
    let client = GameContractClient::new(&env, &contract_id);

    let player1 = Address::generate(&env);
    let wager = 1001; // Exceeds default 1000

    let res = client.try_create_game(&player1, &wager);
    assert!(res.is_err());

    // The error should be StakeLimitExceeded (15)
    // We can check the error code if we want to be precise:
    // let err = res.err().unwrap();
    // assert!(err.get_code() == 15);
}

#[test]
fn test_set_max_stake() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, GameContract);
    let client = GameContractClient::new(&env, &contract_id);

    let issuer = Address::generate(&env);
    let player1 = Address::generate(&env);

    // Setup token
    let stellar_token = env.register_stellar_asset_contract_v2(issuer.clone());
    let token_address = stellar_token.address();
    let stellar_asset_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_address);

    // Mint player balance
    stellar_asset_client.mint(&player1, &1000);

    // Initialize game contract with token
    let admin = Address::generate(&env);
    client.initialize_token(&admin, &token_address);

    // Set limit to 500
    client.set_max_stake(&500);

    // Try to create game with 600
    let res = client.try_create_game(&player1, &600);
    assert!(res.is_err());

    // Try to create game with 500
    let game_id_res = client.try_create_game(&player1, &500);
    assert!(game_id_res.is_ok());
}

#[test]
fn test_payout_with_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, GameContract);
    let client = GameContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let player1 = Address::generate(&env);
    let player2 = Address::generate(&env);
    let treasury_addr = Address::generate(&env);

    // Register token contract
    let stellar_token = env.register_stellar_asset_contract_v2(issuer);
    let token_address = stellar_token.address();
    let stellar_asset_client = StellarAssetClient::new(&env, &token_address);

    // Initialize Game Contract with token
    client.initialize_token(&admin, &token_address);

    // Initialize Puzzle Rewards/Fees
    let admin_key = Bytes::from_slice(&env, &[0u8; 32]);
    client.initialize_puzzle_rewards(&admin, &admin_key, &0i128, &20u32, &treasury_addr); // 2% fee (20 bips)

    let wager = 500; // Total pool 1000
    stellar_asset_client.mint(&player1, &wager);
    stellar_asset_client.mint(&player2, &wager);

    let game_id = client.create_game(&player1, &wager);
    client.join_game(&game_id, &player2);

    // Force complete the game and set winner
    env.as_contract(&contract_id, || {
        let mut games: Map<u64, Game> = env.storage().instance().get(&GAMES).unwrap();
        let mut game = games.get(game_id).unwrap();
        game.state = GameState::Completed;
        game.winner = Some(player1.clone());
        games.set(game_id, game);
        env.storage().instance().set(&GAMES, &games);
    });

    client.payout(&game_id, &player1);

    env.as_contract(&contract_id, || {
        let escrow: Map<Address, i128> = env.storage().instance().get(&ESCROW).unwrap();
        let winner_escrow = escrow.get(player1.clone()).unwrap_or(0);
        let treasury_escrow = escrow.get(treasury_addr.clone()).unwrap_or(0);
        let loser_escrow = escrow.get(player2.clone()).unwrap_or(0);

        assert_eq!(winner_escrow, 980);
        assert_eq!(treasury_escrow, 20);
        assert_eq!(loser_escrow, 0);
    });
}

#[test]
fn test_configure_fees_permissioned() {
    let env = Env::default();
    let contract_id = env.register_contract(None, GameContract);
    let client = GameContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let treasury_addr = Address::generate(&env);
    let admin_key = Bytes::from_slice(&env, &[0u8; 32]);

    env.mock_all_auths();
    client.initialize_puzzle_rewards(&admin, &admin_key, &0i128, &0u32, &treasury_addr);

    // Update fees as admin
    let new_treasury = Address::generate(&env);
    client.configure_fees(&admin, &50, &new_treasury); // 5% fee

    // Verify update
    // (In a real test we'd check storage or run a payout, but here we just ensure it doesn't panic)

    // Attempt update as someone else should panic
    let stranger = Address::generate(&env);
    let res = client.try_configure_fees(&stranger, &100, &new_treasury);
    assert!(res.is_err());
}

#[test]
fn test_upgrade_admin_logic() {
    let env = Env::default();
    let contract_id = env.register_contract(None, GameContract);
    let client = GameContractClient::new(&env, &contract_id);

    let admin_key = Bytes::from_slice(&env, &[0u8; 32]);

    // Manually set ADMIN_KEY to simulate old initialization (pre-CONTRACT_ADMIN)
    env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .set(&symbol_short!("ADMIN_KEY"), &admin_key);
    });

    let admin = Address::generate(&env);
    env.mock_all_auths();

    // upgrade_admin should allow setting the admin for the first time
    client.upgrade_admin(&admin);

    // Further calls to upgrade_admin should panic
    let stranger = Address::generate(&env);
    let res = client.try_upgrade_admin(&stranger);
    assert!(res.is_err());
}
