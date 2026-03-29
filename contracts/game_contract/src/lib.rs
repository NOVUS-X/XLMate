#![no_std]
use soroban_sdk::token::TokenClient;
use soroban_sdk::{
    Address, Env, Map, Symbol, Vec, contract, contracterror, contractimpl, contracttype,
    symbol_short,
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Bytes, BytesN, Env,
    Map, Symbol, Vec,
};

// Game states
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GameState {
    Created,
    InProgress,
    Completed,
    Drawn,
    Forfeited,
}

// Game structure
#[contracttype]
#[derive(Clone, Debug)]
pub struct Game {
    pub id: u64,
    pub player1: Address,
    pub player2: Option<Address>,
    pub state: GameState,
    pub wager_amount: i128,
    pub current_turn: u32, // 1 for player1, 2 for player2
    pub moves: Vec<ChessMove>,
    pub created_at: u64,
    pub winner: Option<Address>,
}

// Move structure
#[contracttype]
#[derive(Clone, Debug)]
pub struct ChessMove {
    pub player: Address,
    pub move_data: Vec<u32>, // Serialized chess move
    pub timestamp: u64,
}

// Contract storage keys
const GAME_COUNTER: Symbol = symbol_short!("GAME_CNT");
const GAMES: Symbol = symbol_short!("GAMES");
const ESCROW: Symbol = symbol_short!("ESCROW");
const TOKEN_CONTRACT: Symbol = symbol_short!("TOKEN");

// Puzzle-reward storage keys
const ADMIN_KEY: Symbol = symbol_short!("ADMIN_KEY"); // 32-byte ED25519 backend pubkey
const TREASURY: Symbol = symbol_short!("TREASURY"); // i128 treasury reserve
const BALANCES: Symbol = symbol_short!("BALANCES"); // Map<Address, i128> user balances
const USED_NONCE: Symbol = symbol_short!("NONCES"); // Map<u64, bool> replay protection

// Multisig upgrade storage key
const UPGRADE_ADMINS: Symbol = symbol_short!("UPG_ADMS");
// Fee storage keys
const FEE_BIPS: Symbol = symbol_short!("FEE_BIPS");
const TREASURY_ADDR: Symbol = symbol_short!("TR_ADDR");
const CONTRACT_ADMIN: Symbol = symbol_short!("CT_ADMIN");

// Contract errors
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum ContractError {
    GameNotFound = 1,
    NotYourTurn = 2,
    GameNotInProgress = 3,
    InvalidMove = 4,
    InsufficientFunds = 5,
    AlreadyJoined = 6,
    GameFull = 7,
    NotPlayer = 8,
    GameAlreadyCompleted = 9,
    DrawNotAvailable = 10,
    ForfeitNotAllowed = 11,
    InvalidPercentage = 12,
    MismatchedLengths = 13,
    InsufficientSignatures = 15,
    InvalidAdmin = 16,
    AdminsNotInitialized = 17,
    /// Returned when an invalid or already-used backend signature is submitted.
    Unauthorized = 14,
}

#[contract]
pub struct GameContract;

#[contractimpl]
impl GameContract {
    pub fn initialize_token(env: Env, admin: Address, token_contract: Address) {
        if env.storage().instance().has(&TOKEN_CONTRACT) {
            panic!("Contract already initialized");
        }
        admin.require_auth();
        env.storage()
            .instance()
            .set(&TOKEN_CONTRACT, &token_contract);
    }

    fn token_contract_address(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&TOKEN_CONTRACT)
            .expect("Token contract is not initialized")
    }

    fn token_client(env: &Env) -> TokenClient {
        TokenClient::new(env, &Self::token_contract_address(env))
    }

    // Create a new game with token-based escrow
    pub fn create_game(
        env: Env,
        player1: Address,
        wager_amount: i128,
    ) -> Result<u64, ContractError> {
        player1.require_auth();

        let token_client = Self::token_client(&env);
        let contract_address = env.current_contract_address();
        let player_balance = token_client.balance(&player1);
        if player_balance < wager_amount {
            return Err(ContractError::InsufficientFunds);
        }

        token_client.transfer(&player1, &contract_address, &wager_amount);

        // Generate unique game ID
        let mut game_counter: u64 = env.storage().instance().get(&GAME_COUNTER).unwrap_or(0);
        game_counter += 1;
        env.storage().instance().set(&GAME_COUNTER, &game_counter);

        // Create new game
        let game = Game {
            id: game_counter,
            player1: player1.clone(),
            player2: None,
            state: GameState::Created,
            wager_amount,
            current_turn: 1,
            moves: Vec::new(&env),
            created_at: env.ledger().sequence() as u64,
            winner: None,
        };

        // Store game
        let mut games: Map<u64, Game> = env
            .storage()
            .instance()
            .get(&GAMES)
            .unwrap_or(Map::new(&env));
        games.set(game_counter, game);
        env.storage().instance().set(&GAMES, &games);

        // Add to escrow
        let mut escrow: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&ESCROW)
            .unwrap_or(Map::new(&env));
        let current_escrow = escrow.get(player1.clone()).unwrap_or(0);
        escrow.set(player1, current_escrow + wager_amount);
        env.storage().instance().set(&ESCROW, &escrow);

        Ok(game_counter)
    }

    // ... (rest of the impl block remains the same until the tests)
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::token::{StellarAssetClient, TokenClient};
    use soroban_sdk::{Address, Env};
    use ed25519_dalek::{Signer, SigningKey};
    use rand::rngs::OsRng;
    use soroban_sdk::{Bytes, BytesN};

    #[test]
    fn test_usdc_staking_workflow() {
        let env = Env::default();
        env.mock_all_auths();

        let issuer = Address::generate(&env);
        let player1 = Address::generate(&env);
        let player2 = Address::generate(&env);

        let stellar_token = env.register_stellar_asset_contract_v2(issuer.clone());
        let token_address = stellar_token.address();
        let token_client = TokenClient::new(&env, &token_address);
        let stellar_asset_client = StellarAssetClient::new(&env, &token_address);

        // Mint both player balances
        let fund_amount: i128 = 1_000;
        stellar_asset_client.mint(&player1, &fund_amount);
        stellar_asset_client.mint(&player2, &fund_amount);

        // Deploy game contract and initialize with token contract
        let contract_id = env.register_contract(None, GameContract);
        let client = GameContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize_token(&admin, &token_address);

        // Player 1 creates game with USDC staking
        let initial_wager: i128 = 100;
        let game_id = client.create_game(&player1, &initial_wager).unwrap();

        // Player 2 joins game
        client.join_game(&game_id, &player2).unwrap();

        // Player 1 forfeits, winner is player 2; contract should pay out 200
        client.forfeit(&game_id, &player1).unwrap();

        let final_player2_balance = token_client.balance(&player2);
        assert_eq!(final_player2_balance, 1_100);
    }

    fn sign_payload(
        env: &Env,
        signing_key: &SigningKey,
        recipient: &Address,
        reward_amount: i128,
        nonce: u64,
    ) -> BytesN<64> {
        let mut payload_bytes = Bytes::new(env);

        let recipient_str = recipient.clone().to_string();
        let str_len = recipient_str.len() as usize;
        let mut addr_buf = [0u8; 64];
        recipient_str.copy_into_slice(&mut addr_buf[..str_len]);
        payload_bytes.append(&Bytes::from_slice(env, &addr_buf[..str_len]));

        let amount_le: [u8; 8] = (reward_amount as i64).to_le_bytes();
        payload_bytes.append(&Bytes::from_slice(env, &amount_le));

        let nonce_le: [u8; 8] = nonce.to_le_bytes();
        payload_bytes.append(&Bytes::from_slice(env, &nonce_le));

        let digest_bytesn: BytesN<32> = env.crypto().sha256(&payload_bytes).into();

        let mut digest_raw = [0u8; 32];
        digest_bytesn.copy_into_slice(&mut digest_raw);

        let dalek_sig = signing_key.sign(&digest_raw);
        BytesN::from_array(env, &dalek_sig.to_bytes())
    }

    fn setup(env: &Env, treasury_amount: i128) -> (GameContractClient<'_>, SigningKey) {
        let contract_id = env.register_contract(None, GameContract);
        let client = GameContractClient::new(env, &contract_id);

        let admin = Address::generate(env);
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key_bytes: [u8; 32] = signing_key.verifying_key().to_bytes();
        let admin_key = Bytes::from_slice(env, &verifying_key_bytes);
        let treasury_addr = Address::generate(env);

        client.initialize_puzzle_rewards(
            &admin,
            &admin_key,
            &treasury_amount,
            &0u32,
            &treasury_addr,
        );
        (client, signing_key)
    }

    #[test]
    fn test_claim_puzzle_reward_valid_sig() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, signing_key) = setup(&env, 10_000);
        let recipient = Address::generate(&env);
        let reward_amount: i128 = 500;
        let nonce: u64 = 1;

        let sig = sign_payload(&env, &signing_key, &recipient, reward_amount, nonce);

        client.claim_puzzle_reward(&recipient, &reward_amount, &nonce, &sig).unwrap();

        assert_eq!(client.reward_balance(&recipient), reward_amount);
        assert_eq!(client.treasury_balance(), 10_000 - reward_amount);
    }

    #[test]
    #[should_panic]
    fn test_claim_puzzle_reward_invalid_sig() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, _signing_key) = setup(&env, 10_000);
        let recipient = Address::generate(&env);

        let wrong_key = SigningKey::generate(&mut OsRng);
        let bad_sig = sign_payload(&env, &wrong_key, &recipient, 500, 1);

        client.claim_puzzle_reward(&recipient, &500, &1, &bad_sig).unwrap();
    }

    #[test]
    fn test_claim_puzzle_reward_replay_rejected() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, signing_key) = setup(&env, 10_000);
        let recipient = Address::generate(&env);
        let reward_amount: i128 = 300;
        let nonce: u64 = 42;

        let sig = sign_payload(&env, &signing_key, &recipient, reward_amount, nonce);

        client.claim_puzzle_reward(&recipient, &reward_amount, &nonce, &sig).unwrap();

        let sig2 = sign_payload(&env, &signing_key, &recipient, reward_amount, nonce);
        let result = client.try_claim_puzzle_reward(&recipient, &reward_amount, &nonce, &sig2);
        assert_eq!(result, Err(Ok(ContractError::Unauthorized)));
    }
}
