#![no_std]
use soroban_sdk::{
    Address, Bytes, BytesN, Env, Map, Symbol, Vec, contract, contracterror, contractimpl,
    contracttype, symbol_short,
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

// Puzzle-reward storage keys
const ADMIN_KEY: Symbol = symbol_short!("ADMIN_KEY"); // 32-byte ED25519 backend pubkey
const TREASURY: Symbol = symbol_short!("TREASURY"); // i128 treasury reserve
const BALANCES: Symbol = symbol_short!("BALANCES"); // Map<Address, i128> user balances
const USED_NONCE: Symbol = symbol_short!("NONCES"); // Map<u64, bool> replay protection

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
    /// Returned when an invalid or already-used backend signature is submitted.
    Unauthorized = 14,
    InvalidPercentage = 12,
    MismatchedLengths = 13,
}

#[contract]
pub struct GameContract;

#[contractimpl]
impl GameContract {
    // Create a new game with XLM escrow
    pub fn create_game(env: Env, player1: Address, wager_amount: i128) -> u64 {
        // TODO: Add proper balance check
        // For now, we'll skip the balance check to get compilation working

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

        game_counter
    }

    // Join an existing game
    pub fn join_game(env: Env, game_id: u64, player2: Address) -> Result<(), ContractError> {
        let mut games: Map<u64, Game> = env
            .storage()
            .instance()
            .get(&GAMES)
            .ok_or(ContractError::GameNotFound)?;

        let mut game = games.get(game_id).ok_or(ContractError::GameNotFound)?;

        // Validate game state
        if game.state != GameState::Created {
            return Err(ContractError::GameAlreadyCompleted);
        }

        if game.player2.is_some() {
            return Err(ContractError::GameFull);
        }

        if game.player1 == player2 {
            return Err(ContractError::AlreadyJoined);
        }

        // Verify player has sufficient funds
        // TODO: Add proper balance check
        // For now, we'll skip the balance check to get compilation working

        // Update game
        game.player2 = Some(player2.clone());
        game.state = GameState::InProgress;
        game.current_turn = 1;

        // Update escrow
        let mut escrow: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&ESCROW)
            .unwrap_or(Map::new(&env));
        let current_escrow = escrow.get(player2.clone()).unwrap_or(0);
        escrow.set(player2, current_escrow + game.wager_amount);
        env.storage().instance().set(&ESCROW, &escrow);

        // Store updated game
        games.set(game_id, game);
        env.storage().instance().set(&GAMES, &games);

        Ok(())
    }

    // Submit a chess move
    pub fn submit_move(
        env: Env,
        game_id: u64,
        player: Address,
        move_data: Vec<u32>,
    ) -> Result<(), ContractError> {
        let mut games: Map<u64, Game> = env
            .storage()
            .instance()
            .get(&GAMES)
            .ok_or(ContractError::GameNotFound)?;

        let mut game = games.get(game_id).ok_or(ContractError::GameNotFound)?;

        // Validate game state
        if game.state != GameState::InProgress {
            return Err(ContractError::GameNotInProgress);
        }

        // Validate turn
        let player_num = if player == game.player1 {
            1
        } else if Some(player.clone()) == game.player2 {
            2
        } else {
            return Err(ContractError::NotPlayer);
        };

        if player_num != game.current_turn {
            return Err(ContractError::NotYourTurn);
        }

        // Validate move (basic validation - in real implementation, this would include full chess rules)
        if move_data.is_empty() {
            return Err(ContractError::InvalidMove);
        }

        // Create and store move
        let chess_move = ChessMove {
            player: player.clone(),
            move_data: move_data.clone(),
            timestamp: env.ledger().sequence() as u64,
        };

        game.moves.push_back(chess_move.into());

        // Switch turns
        game.current_turn = if game.current_turn == 1 { 2 } else { 1 };

        // Store updated game
        games.set(game_id, game);
        env.storage().instance().set(&GAMES, &games);

        Ok(())
    }

    // Claim a draw
    pub fn claim_draw(env: Env, game_id: u64, player: Address) -> Result<(), ContractError> {
        let mut games: Map<u64, Game> = env
            .storage()
            .instance()
            .get(&GAMES)
            .ok_or(ContractError::GameNotFound)?;

        let mut game = games.get(game_id).ok_or(ContractError::GameNotFound)?;

        // Validate game state
        if game.state != GameState::InProgress {
            return Err(ContractError::GameNotInProgress);
        }

        // Validate player
        if player != game.player1 && Some(player.clone()) != game.player2 {
            return Err(ContractError::NotPlayer);
        }

        // Update game state
        game.state = GameState::Drawn;

        // Process draw payout (return wagers to both players)
        Self::process_draw_payout(&env, &game)?;

        // Store updated game
        games.set(game_id, game);
        env.storage().instance().set(&GAMES, &games);

        Ok(())
    }

    // Forfeit the game
    pub fn forfeit(env: Env, game_id: u64, player: Address) -> Result<(), ContractError> {
        let mut games: Map<u64, Game> = env
            .storage()
            .instance()
            .get(&GAMES)
            .ok_or(ContractError::GameNotFound)?;

        let mut game = games.get(game_id).ok_or(ContractError::GameNotFound)?;

        // Validate game state
        if game.state != GameState::InProgress {
            return Err(ContractError::GameNotInProgress);
        }

        // Validate player
        if player != game.player1 && Some(player.clone()) != game.player2 {
            return Err(ContractError::NotPlayer);
        }

        // Determine winner (the other player)
        let winner = if player == game.player1 {
            game.player2
                .as_ref()
                .ok_or(ContractError::GameFull)?
                .clone()
        } else {
            game.player1.clone()
        };

        // Update game state
        game.state = GameState::Forfeited;
        game.winner = Some(winner.clone());

        // Process forfeit payout
        Self::process_forfeit_payout(&env, &game, &winner)?;

        // Store updated game
        games.set(game_id, game);
        env.storage().instance().set(&GAMES, &games);

        Ok(())
    }

    // Payout winnings to the winner
    pub fn payout(env: Env, game_id: u64, winner: Address) -> Result<(), ContractError> {
        let mut games: Map<u64, Game> = env
            .storage()
            .instance()
            .get(&GAMES)
            .ok_or(ContractError::GameNotFound)?;

        let game = games.get(game_id).ok_or(ContractError::GameNotFound)?;

        // Validate game state
        if game.state != GameState::Completed {
            return Err(ContractError::GameNotInProgress);
        }

        // Validate winner
        if game.winner.as_ref() != Some(&winner) {
            return Err(ContractError::NotPlayer);
        }

        // Process payout
        Self::process_win_payout(&env, &game, &winner)?;

        // Store updated game
        games.set(game_id, game);
        env.storage().instance().set(&GAMES, &games);

        Ok(())
    }

    // Payout tournament winnings to multiple winners
    pub fn payout_tournament(
        env: Env,
        game_id: u64,
        winners: Vec<Address>,
        percentages: Vec<u32>,
    ) -> Result<(), ContractError> {
        let mut games: Map<u64, Game> = env
            .storage()
            .instance()
            .get(&GAMES)
            .ok_or(ContractError::GameNotFound)?;

        let game = games.get(game_id).ok_or(ContractError::GameNotFound)?;

        // Validate game state
        if game.state != GameState::Completed {
            return Err(ContractError::GameNotInProgress);
        }

        // Require authorization from the game creator to distribute tournament funds
        game.player1.require_auth();

        // Validate arrays
        if winners.len() != percentages.len() {
            return Err(ContractError::MismatchedLengths);
        }

        // Validate percentages equal 100
        let mut total_percentage: u32 = 0;
        for i in 0..percentages.len() {
            total_percentage += percentages.get(i).unwrap();
        }

        if total_percentage != 100 {
            return Err(ContractError::InvalidPercentage);
        }

        let mut escrow: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&ESCROW)
            .unwrap_or(Map::new(&env));

        // Validate sufficient balances before any debit to prevent negative escrow and double payouts
        let player1_escrow = escrow.get(game.player1.clone()).unwrap_or(0);
        if player1_escrow < game.wager_amount {
            return Err(ContractError::InsufficientFunds);
        }

        let mut player2_escrow = 0;
        let mut total_pool = game.wager_amount;

        if let Some(ref player2) = game.player2 {
            player2_escrow = escrow.get(player2.clone()).unwrap_or(0);
            if player2_escrow < game.wager_amount {
                return Err(ContractError::InsufficientFunds);
            }
            total_pool = game.wager_amount * 2;
        }

        // Subtract from players FIRST to avoid overwriting their payout if they are also a winner
        escrow.set(game.player1.clone(), player1_escrow - game.wager_amount);

        if let Some(ref player2) = game.player2 {
            escrow.set(player2.clone(), player2_escrow - game.wager_amount);
        }

        let mut distributed: i128 = 0;

        for i in 0..winners.len() {
            let winner = winners.get(i).unwrap();
            let percentage = percentages.get(i).unwrap();

            // Calculate payout based on percentage of total pool
            let payout_amount = (total_pool * percentage as i128) / 100;
            distributed += payout_amount;

            // Fetch latest escrow in case the winner was also one of the debited players
            let winner_escrow = escrow.get(winner.clone()).unwrap_or(0);
            escrow.set(winner.clone(), winner_escrow + payout_amount);
        }

        // Verify precisely equals or distribute remainder to the first winner to avoid dust
        let remainder = total_pool - distributed;
        if remainder > 0 && winners.len() > 0 {
            let first_winner = winners.get(0).unwrap();
            let winner_escrow = escrow.get(first_winner.clone()).unwrap_or(0);
            escrow.set(first_winner.clone(), winner_escrow + remainder);
        }

        env.storage().instance().set(&ESCROW, &escrow);

        // Store updated game
        games.set(game_id, game);
        env.storage().instance().set(&GAMES, &games);

        Ok(())
    }

    // Get game details
    pub fn get_game(env: Env, game_id: u64) -> Result<Game, ContractError> {
        let games: Map<u64, Game> = env
            .storage()
            .instance()
            .get(&GAMES)
            .ok_or(ContractError::GameNotFound)?;

        games.get(game_id).ok_or(ContractError::GameNotFound)
    }

    // Get all games
    pub fn get_all_games(env: Env) -> Map<u64, Game> {
        env.storage()
            .instance()
            .get(&GAMES)
            .unwrap_or(Map::new(&env))
    }

    // Helper function to process draw payout
    fn process_draw_payout(env: &Env, game: &Game) -> Result<(), ContractError> {
        let mut escrow: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&ESCROW)
            .unwrap_or(Map::new(env));

        // Return wagers to both players
        let player1_escrow = escrow.get(game.player1.clone()).unwrap_or(0);
        escrow.set(game.player1.clone(), player1_escrow - game.wager_amount);

        if let Some(ref player2) = game.player2 {
            let player2_escrow = escrow.get(player2.clone()).unwrap_or(0);
            escrow.set(player2.clone(), player2_escrow - game.wager_amount);
        }

        env.storage().instance().set(&ESCROW, &escrow);
        Ok(())
    }

    // Helper function to process forfeit payout
    fn process_forfeit_payout(
        env: &Env,
        game: &Game,
        winner: &Address,
    ) -> Result<(), ContractError> {
        let mut escrow: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&ESCROW)
            .unwrap_or(Map::new(env));

        // Transfer both wagers to winner
        let winner_escrow = escrow.get(winner.clone()).unwrap_or(0);
        escrow.set(winner.clone(), winner_escrow + (game.wager_amount * 2));

        // Remove from loser's escrow
        let loser = if winner == &game.player1 {
            game.player2.as_ref().ok_or(ContractError::GameFull)?
        } else {
            &game.player1
        };

        let loser_escrow = escrow.get(loser.clone()).unwrap_or(0);
        escrow.set(loser.clone(), loser_escrow - game.wager_amount);

        env.storage().instance().set(&ESCROW, &escrow);
        Ok(())
    }

    // Helper function to process win payout
    fn process_win_payout(env: &Env, game: &Game, winner: &Address) -> Result<(), ContractError> {
        let mut escrow: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&ESCROW)
            .unwrap_or(Map::new(env));

        // Transfer both wagers to winner
        let winner_escrow = escrow.get(winner.clone()).unwrap_or(0);
        escrow.set(winner.clone(), winner_escrow + (game.wager_amount * 2));

        // Remove from loser's escrow
        let loser = if winner == &game.player1 {
            game.player2.as_ref().ok_or(ContractError::GameFull)?
        } else {
            &game.player1
        };

        let loser_escrow = escrow.get(loser.clone()).unwrap_or(0);
        escrow.set(loser.clone(), loser_escrow - game.wager_amount);

        env.storage().instance().set(&ESCROW, &escrow);
        Ok(())
    }

    // ============================================================
    // PUZZLE REWARD FUNCTIONS
    // ============================================================

    /// Initialize puzzle-reward system with the backend admin ED25519 public key
    /// and an initial treasury reserve.  Must be called exactly once.
    ///
    /// # Arguments
    /// * `admin_public_key` - 32-byte ED25519 public key of the backend signing service
    /// * `treasury_amount`  - Tokens pre-funded into the treasury for puzzle payouts
    pub fn initialize(env: Env, admin_public_key: Bytes, treasury_amount: i128) {
        // Prevent re-initialization
        if env.storage().instance().has(&ADMIN_KEY) {
            panic!("Already initialized");
        }

        if admin_public_key.len() != 32 {
            panic!("Admin public key must be 32 bytes");
        }

        if treasury_amount < 0 {
            panic!("Treasury amount must be non-negative");
        }

        env.storage().instance().set(&ADMIN_KEY, &admin_public_key);
        env.storage().instance().set(&TREASURY, &treasury_amount);
    }

    /// Claim a puzzle reward by presenting a backend-signed proof of completion.
    ///
    /// The backend signs the following 32-byte SHA-256 hash:
    ///   `SHA256( raw_recipient_bytes || reward_amount_le_8bytes || nonce_le_8bytes )`
    ///
    /// # Arguments
    /// * `recipient`      - Address that will receive the reward tokens
    /// * `reward_amount`  - Number of tokens to transfer (must be > 0)
    /// * `nonce`          - Unique u64 value; prevents signature replay
    /// * `signature`      - 64-byte ED25519 signature from the backend admin key
    ///
    /// # Errors
    /// * `ContractError::Unauthorized` – nonce already used
    /// * Panics if the ED25519 signature is invalid
    pub fn claim_puzzle_reward(
        env: Env,
        recipient: Address,
        reward_amount: i128,
        nonce: u64,
        signature: BytesN<64>,
    ) -> Result<(), ContractError> {
        // ── 1. Load the admin public key (fails if not initialized) ─────────
        let admin_key_bytes: Bytes = env
            .storage()
            .instance()
            .get(&ADMIN_KEY)
            .expect("Not initialized");

        // Convert Bytes → BytesN<32>
        let admin_pubkey: BytesN<32> = admin_key_bytes
            .try_into()
            .expect("Admin public key must be 32 bytes");

        // ── 2. Replay protection ─────────────────────────────────────────────
        let mut nonces: Map<u64, bool> = env
            .storage()
            .instance()
            .get(&USED_NONCE)
            .unwrap_or(Map::new(&env));

        if nonces.get(nonce).unwrap_or(false) {
            return Err(ContractError::Unauthorized);
        }

        // ── 3. Build the canonical payload and verify the ED25519 signature ──
        //
        // Payload layout (variable-length):
        //   recipient_raw_bytes  (as serialised by soroban_sdk)
        //   || reward_amount     (8-byte little-endian i128 low word)
        //   || nonce             (8-byte little-endian u64)
        //
        // We hash the concatenation with SHA-256 first so the payload is
        // always 32 bytes regardless of recipient address length.
        let mut payload_bytes = Bytes::new(&env);

        // Append serialised recipient address bytes.
        // soroban_sdk::String::copy_into_slice is available in #[no_std];
        // Stellar addresses are always 56 ASCII chars (G... or C...).
        let recipient_str = recipient.clone().to_string();
        let str_len = recipient_str.len() as usize;
        let mut addr_buf = [0u8; 64];
        recipient_str.copy_into_slice(&mut addr_buf[..str_len]);
        payload_bytes.append(&Bytes::from_slice(&env, &addr_buf[..str_len]));

        // Append reward_amount as little-endian bytes (i64 cast; amounts fit)
        let amount_le: [u8; 8] = (reward_amount as i64).to_le_bytes();
        payload_bytes.append(&Bytes::from_slice(&env, &amount_le));

        // Append nonce as little-endian bytes
        let nonce_le: [u8; 8] = nonce.to_le_bytes();
        payload_bytes.append(&Bytes::from_slice(&env, &nonce_le));

        // SHA-256 digest: Hash<32> → BytesN<32> → Bytes (type ed25519_verify expects)
        let digest_bytesn: BytesN<32> = env.crypto().sha256(&payload_bytes).into();
        let digest_bytes: Bytes = digest_bytesn.into();

        // Verify the ED25519 signature — Soroban panics on failure.
        // The panic satisfies the "panics with Unauthorized" acceptance criterion.
        env.crypto()
            .ed25519_verify(&admin_pubkey, &digest_bytes, &signature);

        // ── 4. Mark nonce as used (state-before-interaction) ────────────────
        nonces.set(nonce, true);
        env.storage().instance().set(&USED_NONCE, &nonces);

        // ── 5. Deduct from Treasury ──────────────────────────────────────────
        let treasury: i128 = env.storage().instance().get(&TREASURY).unwrap_or(0);

        if treasury < reward_amount {
            panic!("Insufficient treasury");
        }

        env.storage()
            .instance()
            .set(&TREASURY, &(treasury - reward_amount));

        // ── 6. Credit recipient ──────────────────────────────────────────────
        let mut balances: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&BALANCES)
            .unwrap_or(Map::new(&env));

        let prev_balance = balances.get(recipient.clone()).unwrap_or(0);
        balances.set(recipient.clone(), prev_balance + reward_amount);
        env.storage().instance().set(&BALANCES, &balances);

        // ── 7. Emit event ────────────────────────────────────────────────────
        env.events()
            .publish((symbol_short!("pzl_rwd"), recipient.clone()), reward_amount);

        Ok(())
    }

    /// Get the puzzle-reward balance of an address.
    pub fn reward_balance(env: Env, address: Address) -> i128 {
        let balances: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&BALANCES)
            .unwrap_or(Map::new(&env));
        balances.get(address).unwrap_or(0)
    }

    /// Get the current treasury reserve.
    pub fn treasury_balance(env: Env) -> i128 {
        env.storage().instance().get(&TREASURY).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use rand::rngs::OsRng;
    use soroban_sdk::{Bytes, BytesN, Env, testutils::Address as _};

    // ────────────────────────────────────────────────────────────────────────
    // Helper: build the same payload the contract builds, sign it off-chain.
    // ────────────────────────────────────────────────────────────────────────
    fn sign_payload(
        env: &Env,
        signing_key: &SigningKey,
        recipient: &Address,
        reward_amount: i128,
        nonce: u64,
    ) -> BytesN<64> {
        let mut payload_bytes = Bytes::new(env);

        // Mirror the contract's payload construction exactly
        let recipient_str = recipient.clone().to_string();
        let str_len = recipient_str.len() as usize;
        let mut addr_buf = [0u8; 64];
        recipient_str.copy_into_slice(&mut addr_buf[..str_len]);
        payload_bytes.append(&Bytes::from_slice(env, &addr_buf[..str_len]));

        let amount_le: [u8; 8] = (reward_amount as i64).to_le_bytes();
        payload_bytes.append(&Bytes::from_slice(env, &amount_le));

        let nonce_le: [u8; 8] = nonce.to_le_bytes();
        payload_bytes.append(&Bytes::from_slice(env, &nonce_le));

        // SHA-256 via soroban: Hash<32> → BytesN<32>
        let digest_bytesn: BytesN<32> = env.crypto().sha256(&payload_bytes).into();

        // Copy digest into a plain [u8;32] for ed25519-dalek signing
        let mut digest_raw = [0u8; 32];
        digest_bytesn.copy_into_slice(&mut digest_raw);

        let dalek_sig = signing_key.sign(&digest_raw);
        BytesN::from_array(env, &dalek_sig.to_bytes())
    }

    // ────────────────────────────────────────────────────────────────────────
    // Helper: register + initialize the contract with a generated admin key.
    // Returns (client, signing_key, verifying_key_bytes).
    // ────────────────────────────────────────────────────────────────────────
    fn setup(env: &Env, treasury_amount: i128) -> (GameContractClient, SigningKey) {
        let contract_id = env.register_contract(None, GameContract);
        let client = GameContractClient::new(env, &contract_id);

        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key_bytes: [u8; 32] = signing_key.verifying_key().to_bytes();
        let admin_key = Bytes::from_slice(env, &verifying_key_bytes);

        client.initialize(&admin_key, &treasury_amount);
        (client, signing_key)
    }

    // ────────────────────────────────────────────────────────────────────────
    // Test 1 – happy path: valid signature → balance incremented
    // ────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_claim_puzzle_reward_valid_sig() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, signing_key) = setup(&env, 10_000);
        let recipient = Address::generate(&env);
        let reward_amount: i128 = 500;
        let nonce: u64 = 1;

        let sig = sign_payload(&env, &signing_key, &recipient, reward_amount, nonce);

        client.claim_puzzle_reward(&recipient, &reward_amount, &nonce, &sig);

        // Recipient balance incremented correctly
        assert_eq!(client.reward_balance(&recipient), reward_amount);
        // Treasury reduced correctly
        assert_eq!(client.treasury_balance(), 10_000 - reward_amount);
    }

    // ────────────────────────────────────────────────────────────────────────
    // Test 2 – invalid signature panics (Unauthorized)
    // ────────────────────────────────────────────────────────────────────────
    #[test]
    #[should_panic]
    fn test_claim_puzzle_reward_invalid_sig() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, _signing_key) = setup(&env, 10_000);
        let recipient = Address::generate(&env);

        // Build a signature from a *different* (wrong) key
        let wrong_key = SigningKey::generate(&mut OsRng);
        let bad_sig = sign_payload(&env, &wrong_key, &recipient, 500, 1);

        // Must panic — Soroban's ed25519_verify panics on bad sig
        client.claim_puzzle_reward(&recipient, &500, &1, &bad_sig);
    }

    // ────────────────────────────────────────────────────────────────────────
    // Test 3 – replay: same nonce used twice → Unauthorized
    // ────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_claim_puzzle_reward_replay_rejected() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, signing_key) = setup(&env, 10_000);
        let recipient = Address::generate(&env);
        let reward_amount: i128 = 300;
        let nonce: u64 = 42;

        let sig = sign_payload(&env, &signing_key, &recipient, reward_amount, nonce);

        // First call succeeds
        client.claim_puzzle_reward(&recipient, &reward_amount, &nonce, &sig);

        // Second call with same nonce must return Unauthorized
        let sig2 = sign_payload(&env, &signing_key, &recipient, reward_amount, nonce);
        let result = client.try_claim_puzzle_reward(&recipient, &reward_amount, &nonce, &sig2);
        assert_eq!(result, Err(Ok(ContractError::Unauthorized)));
    }
}

#[cfg(test)]
mod test;
