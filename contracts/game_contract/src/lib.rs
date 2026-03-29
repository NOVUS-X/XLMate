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
    pub fn initialize(env: Env, token_contract: Address) {
        if env.storage().instance().has(&TOKEN_CONTRACT) {
            panic!("Contract already initialized");
        }
        token_contract.require_auth();
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
    pub fn create_game(env: Env, player1: Address, wager_amount: i128) -> u64 {
    // FIX 3: Changed `panic!("Insufficient funds")` to return
    // `Err(ContractError::InsufficientFunds)` for consistent API behavior
    // with `join_game` which already uses the Result-based error pattern.
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
            panic!("Insufficient funds");
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

        player2.require_auth();
        let token_client = Self::token_client(&env);
        let contract_address = env.current_contract_address();
        let player2_balance = token_client.balance(&player2);
        if player2_balance < game.wager_amount {
            return Err(ContractError::InsufficientFunds);
        }

        token_client.transfer(&player2, &contract_address, &game.wager_amount);

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

        // Distribute any integer-division remainder to the first winner to avoid dust
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
        let token_client = Self::token_client(env);
        let contract_address = env.current_contract_address();

        token_client.transfer(&contract_address, &game.player1, &game.wager_amount);

        let mut escrow: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&ESCROW)
            .unwrap_or(Map::new(env));

        // Return wagers to both players
        let player1_escrow = escrow.get(game.player1.clone()).unwrap_or(0);
        escrow.set(game.player1.clone(), player1_escrow - game.wager_amount);

        if let Some(ref player2) = game.player2 {
            token_client.transfer(&contract_address, player2, &game.wager_amount);
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
        let token_client = Self::token_client(env);
        let contract_address = env.current_contract_address();
        token_client.transfer(&contract_address, winner, &(game.wager_amount * 2));

        let mut escrow: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&ESCROW)
            .unwrap_or(Map::new(env));
        let winner_escrow = escrow.get(winner.clone()).unwrap_or(0);
        escrow.set(winner.clone(), winner_escrow + (game.wager_amount * 2));


        let fee_bips: u32 = env.storage().instance().get(&FEE_BIPS).unwrap_or(0);
        let treasury_addr_opt: Option<Address> = env.storage().instance().get(&TREASURY_ADDR);

        let total_pool = game.wager_amount * 2;

        let (payout, fee) = if treasury_addr_opt.is_some() {
            let fee = (total_pool * fee_bips as i128) / 1000;
            (total_pool - fee, fee)
        } else {
            (total_pool, 0)
        };

        // Subtract wagers from both players first (clean state)
        let player1_escrow = escrow.get(game.player1.clone()).unwrap_or(0);
        escrow.set(game.player1.clone(), player1_escrow - game.wager_amount);

        let player2 = game.player2.as_ref().ok_or(ContractError::GameFull)?;
        let player2_escrow = escrow.get(player2.clone()).unwrap_or(0);
        escrow.set(player2.clone(), player2_escrow - game.wager_amount);

        // Add payout to winner
        let winner_escrow = escrow.get(winner.clone()).unwrap_or(0);
        escrow.set(winner.clone(), winner_escrow + payout);

        // Add fee to treasury if it exists
        if fee > 0 {
            if let Some(ref treasury_addr) = treasury_addr_opt {
                let treasury_escrow = escrow.get(treasury_addr.clone()).unwrap_or(0);
                escrow.set(treasury_addr.clone(), treasury_escrow + fee);
            }
        }

        env.storage().instance().set(&ESCROW, &escrow);

        // Perform physical token transfers
        let token_client = Self::token_client(env);
        let contract_address = env.current_contract_address();
        
        token_client.transfer(&contract_address, winner, &payout);
        if fee > 0 {
            if let Some(ref treasury_addr) = treasury_addr_opt {
                token_client.transfer(&contract_address, treasury_addr, &fee);
            }
        }

        Ok(())
    }

    // Helper function to process win payout
    fn process_win_payout(env: &Env, game: &Game, winner: &Address) -> Result<(), ContractError> {
        let token_client = Self::token_client(env);
        let contract_address = env.current_contract_address();
        token_client.transfer(&contract_address, winner, &(game.wager_amount * 2));

        let mut escrow: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&ESCROW)
            .unwrap_or(Map::new(env));
        let winner_escrow = escrow.get(winner.clone()).unwrap_or(0);
        escrow.set(winner.clone(), winner_escrow + (game.wager_amount * 2));


        let fee_bips: u32 = env.storage().instance().get(&FEE_BIPS).unwrap_or(0);
        let treasury_addr_opt: Option<Address> = env.storage().instance().get(&TREASURY_ADDR);

        let total_pool = game.wager_amount * 2;

        let (payout, fee) = if treasury_addr_opt.is_some() {
            let fee = (total_pool * fee_bips as i128) / 1000;
            (total_pool - fee, fee)
        } else {
            (total_pool, 0)
        };

        // Subtract wagers from both players first (clean state)
        let player1_escrow = escrow.get(game.player1.clone()).unwrap_or(0);
        escrow.set(game.player1.clone(), player1_escrow - game.wager_amount);

        let player2 = game.player2.as_ref().ok_or(ContractError::GameFull)?;
        let player2_escrow = escrow.get(player2.clone()).unwrap_or(0);
        escrow.set(player2.clone(), player2_escrow - game.wager_amount);

        // Add payout to winner
        let winner_escrow = escrow.get(winner.clone()).unwrap_or(0);
        escrow.set(winner.clone(), winner_escrow + payout);

        // Add fee to treasury if it exists
        if fee > 0 {
            if let Some(ref treasury_addr) = treasury_addr_opt {
                let treasury_escrow = escrow.get(treasury_addr.clone()).unwrap_or(0);
                escrow.set(treasury_addr.clone(), treasury_escrow + fee);
            }
        }

        env.storage().instance().set(&ESCROW, &escrow);

        // Perform physical token transfers
        let token_client = Self::token_client(env);
        let contract_address = env.current_contract_address();
        
        token_client.transfer(&contract_address, winner, &payout);
        if fee > 0 {
            if let Some(ref treasury_addr) = treasury_addr_opt {
                token_client.transfer(&contract_address, treasury_addr, &fee);
            }
        }

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
    // FIX 1 (continued): Renamed from `initialize` to `initialize_puzzle_rewards`
    // to resolve the duplicate function name compilation error.
    pub fn initialize_puzzle_rewards(
        env: Env,
        admin: Address,
        admin_public_key: Bytes,
        treasury_amount: i128,
        fee_bips: u32,
        treasury_address: Address,
    ) {
        // Prevent re-initialization
        if env.storage().instance().has(&CONTRACT_ADMIN) {
            panic!("Already initialized");
        }

        admin.require_auth();

        if admin_public_key.len() != 32 {
            panic!("Admin public key must be 32 bytes");
        }

        if treasury_amount < 0 {
            panic!("Treasury amount must be non-negative");
        }

        if fee_bips > 1000 {
            panic!("Fee bips must be between 0 and 1000");
        }

        env.storage().instance().set(&CONTRACT_ADMIN, &admin);
        env.storage().instance().set(&ADMIN_KEY, &admin_public_key);
        env.storage().instance().set(&TREASURY, &treasury_amount);
        env.storage().instance().set(&FEE_BIPS, &fee_bips);
        env.storage().instance().set(&TREASURY_ADDR, &treasury_address);
    }

    /// Update fee configuration. Only callable by the contract admin.
    pub fn configure_fees(env: Env, admin: Address, fee_bips: u32, treasury_address: Address) {
        let current_admin: Address = env
            .storage()
            .instance()
            .get(&CONTRACT_ADMIN)
            .expect("Not initialized");
        current_admin.require_auth();

        if admin != current_admin {
            panic!("Unauthorized admin address");
        }

        if fee_bips > 1000 {
            panic!("Fee bips must be between 0 and 1000");
        }

        env.storage().instance().set(&FEE_BIPS, &fee_bips);
        env.storage().instance().set(&TREASURY_ADDR, &treasury_address);
    }

    /// Support legacy deployments by allowing the first admin to be set if missing.
    pub fn upgrade_admin(env: Env, admin: Address) {
        if env.storage().instance().has(&CONTRACT_ADMIN) {
            panic!("Admin already set");
        }

        // For upgrade safety, we can only do this if it was previously initialized but without an admin
        if !env.storage().instance().has(&ADMIN_KEY) {
            panic!("Contract must be initialized first");
        }

        admin.require_auth();
        env.storage().instance().set(&CONTRACT_ADMIN, &admin);
    }

    // ============================================================
    // MULTISIG UPGRADE FUNCTIONS
    // ============================================================

    /// Initialize the 3 administrator addresses for contract upgrades.
    /// Can only be called once.
    pub fn init_upgrade_admins(env: Env, admin1: Address, admin2: Address, admin3: Address) {
        if env.storage().instance().has(&UPGRADE_ADMINS) {
            panic!("Upgrade admins already initialized");
        }
        let mut admins = Vec::new(&env);
        admins.push_back(admin1);
        admins.push_back(admin2);
        admins.push_back(admin3);
        env.storage().instance().set(&UPGRADE_ADMINS, &admins);
    }

    /// Upgrade the contract WASM.
    /// Requires authorization from at least 2 of the 3 configured admins.
    pub fn upgrade(
        env: Env,
        new_wasm_hash: BytesN<32>,
        signers: Vec<Address>,
    ) -> Result<(), ContractError> {
        let admins: Vec<Address> = env
            .storage()
            .instance()
            .get(&UPGRADE_ADMINS)
            .ok_or(ContractError::AdminsNotInitialized)?;

        // Ensure we have at least 2 signers
        if signers.len() < 2 {
            return Err(ContractError::InsufficientSignatures);
        }

        let admin1 = admins.get(0).unwrap();
        let admin2 = admins.get(1).unwrap();
        let admin3 = admins.get(2).unwrap();

        let mut valid_signatures = 0;
        let mut seen_admin1 = false;
        let mut seen_admin2 = false;
        let mut seen_admin3 = false;

        for i in 0..signers.len() {
            let signer = signers.get(i).unwrap();
            
            if signer == admin1 {
                if seen_admin1 { continue; }
                seen_admin1 = true;
            } else if signer == admin2 {
                if seen_admin2 { continue; }
                seen_admin2 = true;
            } else if signer == admin3 {
                if seen_admin3 { continue; }
                seen_admin3 = true;
            } else {
                return Err(ContractError::InvalidAdmin);
            }

            // Ensure the signer has authorized this call
            signer.require_auth();
            valid_signatures += 1;
        }

        if valid_signatures < 2 {
            return Err(ContractError::InsufficientSignatures);
        }

        // Perform the upgrade
        env.deployer().update_current_contract_wasm(new_wasm_hash);
        Ok(())
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
        let mut payload_bytes = Bytes::new(&env);

        let recipient_str = recipient.clone().to_string();
        let str_len = recipient_str.len() as usize;
        let mut addr_buf = [0u8; 64];
        recipient_str.copy_into_slice(&mut addr_buf[..str_len]);
        payload_bytes.append(&Bytes::from_slice(&env, &addr_buf[..str_len]));

        let amount_le: [u8; 8] = (reward_amount as i64).to_le_bytes();
        payload_bytes.append(&Bytes::from_slice(&env, &amount_le));

        let nonce_le: [u8; 8] = nonce.to_le_bytes();
        payload_bytes.append(&Bytes::from_slice(&env, &nonce_le));

        let digest_bytesn: BytesN<32> = env.crypto().sha256(&payload_bytes).into();
        let digest_bytes: Bytes = digest_bytesn.into();

        // Verify the ED25519 signature — Soroban panics on failure.
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

// FIX 4: The original mod tests block was closed prematurely at line 774,
// orphaning the puzzle-reward test helpers and tests (sign_payload, setup,
// test_claim_puzzle_reward_valid_sig, etc.) outside the module. All tests
// are now contained within a single #[cfg(test)] mod tests block.
#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::token::{StellarAssetClient, TokenClient};
    use soroban_sdk::{Address, Env};

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
        client.initialize(&token_address);

        // Player 1 creates game with USDC staking
        let initial_wager: i128 = 100;
        let game_id = client.create_game(&player1, &initial_wager);

        // Player 2 joins game
        client.join_game(&game_id, &player2);

        // Player 1 forfeits, winner is player 2; contract should pay out 200
        client.forfeit(&game_id, &player1);

        let final_player2_balance = token_client.balance(&player2);
        assert_eq!(final_player2_balance, 1_100);
    }
}
    use soroban_sdk::{
        testutils::Address as _,
        Bytes, BytesN, Env,
    };
    use ed25519_dalek::{Signer, SigningKey};
    use rand::rngs::OsRng;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::token::{StellarAssetClient, TokenClient};
    use soroban_sdk::{Address, Bytes, BytesN, Env};

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

        // Updated to match new initialize_token signature (admin, token_contract)
        let admin = Address::generate(&env);
        client.initialize_token(&admin, &token_address);

        // Player 1 creates game with USDC staking
        let initial_wager: i128 = 100;
        let game_id = client.create_game(&player1, &initial_wager);

        // Player 2 joins game
        client.join_game(&game_id, &player2);

        // Player 1 forfeits, winner is player 2; contract should pay out 200
        client.forfeit(&game_id, &player1);

        let final_player2_balance = token_client.balance(&player2);
        assert_eq!(final_player2_balance, 1_100);
    }

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

    // ────────────────────────────────────────────────────────────────────────
    // Helper: register + initialize the contract with a generated admin key.
    // Returns (client, signing_key).
    // ────────────────────────────────────────────────────────────────────────
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

        assert_eq!(client.reward_balance(&recipient), reward_amount);
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

        let wrong_key = SigningKey::generate(&mut OsRng);
        let bad_sig = sign_payload(&env, &wrong_key, &recipient, 500, 1);

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

        client.claim_puzzle_reward(&recipient, &reward_amount, &nonce, &sig);

        let sig2 = sign_payload(&env, &signing_key, &recipient, reward_amount, nonce);
        let result = client.try_claim_puzzle_reward(&recipient, &reward_amount, &nonce, &sig2);
        assert_eq!(result, Err(Ok(ContractError::Unauthorized)));
    }
}

#[cfg(test)]
mod test;
