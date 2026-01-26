use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

// Client message types
#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ClientMessage {
    JoinRoom(JoinRoomPayload),
    SendMove(SendMovePayload),
    LeaveRoom(LeaveRoomPayload),
    RequestGameLog(RequestGameLogPayload),
    OfferTakeback(OfferTakebackPayload),
    AcceptTakeback(AcceptTakebackPayload),
    RejectTakeback(RejectTakebackPayload),
}

#[derive(Debug, Deserialize)]
pub struct JoinRoomPayload {
    pub room_id: String,
    pub player_id: String,
    pub player_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SendMovePayload {
    pub room_id: String,
    pub player_id: String,
    pub move_notation: String,
}

#[derive(Debug, Deserialize)]
pub struct LeaveRoomPayload {
    pub room_id: String,
    pub player_id: String,
}

#[derive(Debug, Deserialize)]
pub struct RequestGameLogPayload {
    pub room_id: String,
}

#[derive(Debug, Deserialize)]
pub struct OfferTakebackPayload {
    pub room_id: String,
    pub player_id: String,
}

#[derive(Debug, Deserialize)]
pub struct AcceptTakebackPayload {
    pub room_id: String,
    pub player_id: String,
}

#[derive(Debug, Deserialize)]
pub struct RejectTakebackPayload {
    pub room_id: String,
    pub player_id: String,
}

// Server message types
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    RoomJoined {
        room_id: String,
        player_id: String,
        players: Vec<Player>,
        game_state: Option<GameState>,
    },
    MoveMade {
        room_id: String,
        player_id: String,
        move_notation: String,
        game_state: GameState,
    },
    PlayerLeft {
        room_id: String,
        player_id: String,
    },
    GameLog {
        room_id: String,
        moves: Vec<MoveRecord>,
    },
    TakebackOffered {
        room_id: String,
        requester_id: String,
    },
    TakebackAccepted {
        room_id: String,
        game_state: GameState,
        moves: Vec<MoveRecord>,
    },
    TakebackRejected {
        room_id: String,
        by_player_id: String,
    },
    Error {
        code: String,
        message: String,
    },
}

// Game state models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: String,
    pub name: String,
    pub color: Option<PieceColor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PieceColor {
    White,
    Black,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub board: HashMap<String, ChessPiece>,
    pub current_turn: PieceColor,
    pub status: GameStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChessPiece {
    pub piece_type: PieceType,
    pub color: PieceColor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PieceType {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameStatus {
    Waiting,
    InProgress,
    Checkmate,
    Stalemate,
    Draw,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveRecord {
    pub player_id: String,
    pub move_notation: String,
    pub timestamp: u64,
}

impl MoveRecord {
    pub fn new(player_id: String, move_notation: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        Self {
            player_id,
            move_notation,
            timestamp,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: String,
    pub players: Vec<Player>,
    pub game_state: Option<GameState>,
    pub moves: Vec<MoveRecord>,
    pub pending_takeback: Option<String>,
}

impl Room {
    pub fn new(id: String) -> Self {
        Self {
            id,
            players: Vec::new(),
            game_state: None,
            moves: Vec::new(),
            pending_takeback: None,
        }
    }
    
    pub fn add_player(&mut self, player: Player) -> Result<(), String> {
        if self.players.len() >= 2 {
            return Err("Room is full".to_string());
        }
        
        // Check if player is already in the room
        if self.players.iter().any(|p| p.id == player.id) {
            return Err("Player is already in the room".to_string());
        }
        
        // Assign color if this is the first or second player
        let mut player = player;
        if self.players.is_empty() {
            player.color = Some(PieceColor::White);
        } else if self.players.len() == 1 {
            player.color = Some(PieceColor::Black);
            
            // Initialize game state when second player joins
            self.game_state = Some(GameState::new_game());
        }
        
        self.players.push(player);
        Ok(())
    }
    
    pub fn remove_player(&mut self, player_id: &str) -> bool {
        let initial_len = self.players.len();
        self.players.retain(|p| p.id != player_id);
        initial_len != self.players.len()
    }
    
    pub fn add_move(&mut self, player_id: String, move_notation: String) {
        let move_record = MoveRecord::new(player_id, move_notation);
        self.moves.push(move_record);
    }
}

impl GameState {
    pub fn new_game() -> Self {
        // Initialize a standard chess board
        let mut board = HashMap::new();
        
        // Set up pawns
        for file in "abcdefgh".chars() {
            let white_pawn_pos = format!("{}{}", file, 2);
            let black_pawn_pos = format!("{}{}", file, 7);
            
            board.insert(white_pawn_pos, ChessPiece { piece_type: PieceType::Pawn, color: PieceColor::White });
            board.insert(black_pawn_pos, ChessPiece { piece_type: PieceType::Pawn, color: PieceColor::Black });
        }
        
        // Set up other pieces
        for (file, piece_type) in "abcdefgh".chars().zip([
            PieceType::Rook, PieceType::Knight, PieceType::Bishop, PieceType::Queen,
            PieceType::King, PieceType::Bishop, PieceType::Knight, PieceType::Rook
        ].iter()) {
            let white_pos = format!("{}{}", file, 1);
            let black_pos = format!("{}{}", file, 8);
            
            board.insert(white_pos, ChessPiece { piece_type: piece_type.clone(), color: PieceColor::White });
            board.insert(black_pos, ChessPiece { piece_type: piece_type.clone(), color: PieceColor::Black });
        }
        
        Self {
            board,
            current_turn: PieceColor::White,
            status: GameStatus::InProgress,
        }
    }
    
    // Apply a move to the game state
    // This is a simplified implementation that doesn't validate chess rules
    pub fn apply_move(&mut self, move_notation: &str) -> Result<(), String> {
        // In a real implementation, this would parse the move notation and update the board
        // For now, we'll just toggle the current turn
        
        self.current_turn = match self.current_turn {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
        };
        
        Ok(())
    }
}
