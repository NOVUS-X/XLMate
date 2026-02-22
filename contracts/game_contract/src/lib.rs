#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, contracterror, symbol_short, Address, Env, Symbol, Vec, Map};

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
        let mut games: Map<u64, Game> = env.storage().instance().get(&GAMES).unwrap_or(Map::new(&env));
        games.set(game_counter, game);
        env.storage().instance().set(&GAMES, &games);

        // Add to escrow
        let mut escrow: Map<Address, i128> = env.storage().instance().get(&ESCROW).unwrap_or(Map::new(&env));
        let current_escrow = escrow.get(player1.clone()).unwrap_or(0);
        escrow.set(player1, current_escrow + wager_amount);
        env.storage().instance().set(&ESCROW, &escrow);

        game_counter
    }

    // Join an existing game
    pub fn join_game(env: Env, game_id: u64, player2: Address) -> Result<(), ContractError> {
        let mut games: Map<u64, Game> = env.storage().instance().get(&GAMES)
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
        let mut escrow: Map<Address, i128> = env.storage().instance().get(&ESCROW).unwrap_or(Map::new(&env));
        let current_escrow = escrow.get(player2.clone()).unwrap_or(0);
        escrow.set(player2, current_escrow + game.wager_amount);
        env.storage().instance().set(&ESCROW, &escrow);

        // Store updated game
        games.set(game_id, game);
        env.storage().instance().set(&GAMES, &games);

        Ok(())
    }

    // Submit a chess move
    pub fn submit_move(env: Env, game_id: u64, player: Address, move_data: Vec<u32>) -> Result<(), ContractError> {
        let mut games: Map<u64, Game> = env.storage().instance().get(&GAMES)
            .ok_or(ContractError::GameNotFound)?;
        
        let mut game = games.get(game_id).ok_or(ContractError::GameNotFound)?;

        // Validate game state
        if game.state != GameState::InProgress {
            return Err(ContractError::GameNotInProgress);
        }

        // Validate turn
        let player_num = if player == game.player1 { 1 } else if Some(player.clone()) == game.player2 { 2 } else {
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
        let mut games: Map<u64, Game> = env.storage().instance().get(&GAMES)
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
        let mut games: Map<u64, Game> = env.storage().instance().get(&GAMES)
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
            game.player2.as_ref().ok_or(ContractError::GameFull)?.clone()
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
        let mut games: Map<u64, Game> = env.storage().instance().get(&GAMES)
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

    // Get game details
    pub fn get_game(env: Env, game_id: u64) -> Result<Game, ContractError> {
        let games: Map<u64, Game> = env.storage().instance().get(&GAMES)
            .ok_or(ContractError::GameNotFound)?;
        
        games.get(game_id).ok_or(ContractError::GameNotFound)
    }

    // Get all games
    pub fn get_all_games(env: Env) -> Map<u64, Game> {
        env.storage().instance().get(&GAMES).unwrap_or(Map::new(&env))
    }

    // Helper function to process draw payout
    fn process_draw_payout(env: &Env, game: &Game) -> Result<(), ContractError> {
        let mut escrow: Map<Address, i128> = env.storage().instance().get(&ESCROW).unwrap_or(Map::new(env));
        
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
    fn process_forfeit_payout(env: &Env, game: &Game, winner: &Address) -> Result<(), ContractError> {
        let mut escrow: Map<Address, i128> = env.storage().instance().get(&ESCROW).unwrap_or(Map::new(env));
        
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
        let mut escrow: Map<Address, i128> = env.storage().instance().get(&ESCROW).unwrap_or(Map::new(env));
        
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
}

