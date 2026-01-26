use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::models::{GameState, MoveRecord, Player, Room, ServerMessage};

// Type alias for the broadcast sender
type MessageSender = broadcast::Sender<ServerMessage>;

// Global game state
lazy_static::lazy_static! {
    static ref GAME_STATE: Arc<Mutex<GameState>> = Arc::new(Mutex::new(GameState {
        rooms: HashMap::new(),
        message_senders: HashMap::new(),
    }));
}

// Initialize the game state
pub fn init_game_state() {
    // This function is called at startup to ensure the lazy_static is initialized
    let _guard = GAME_STATE.lock().unwrap();
    log::info!("Game state initialized");
}

// Get a clone of the message sender for a room
pub fn get_room_sender(room_id: &str) -> Option<MessageSender> {
    let state = GAME_STATE.lock().unwrap();
    state.message_senders.get(room_id).cloned()
}

// Create a new room
pub fn create_room() -> String {
    let room_id = Uuid::new_v4().to_string();
    let (tx, _) = broadcast::channel(100); // Buffer size of 100 messages
    
    let mut state = GAME_STATE.lock().unwrap();
    state.rooms.insert(room_id.clone(), Room::new(room_id.clone()));
    state.message_senders.insert(room_id.clone(), tx);
    
    room_id
}

// Join a room
pub fn join_room(room_id: &str, player_id: &str, player_name: Option<String>) -> Result<ServerMessage, String> {
    let mut state = GAME_STATE.lock().unwrap();
    
    // Check if room exists, create if not
    if !state.rooms.contains_key(room_id) {
                drop(state); // Release the lock before calling create_room
                let _ = create_room(); // This creates a new room with a UUID
                state = GAME_STATE.lock().unwrap();
                // Now create the room with the requested ID
                let (tx, _) = broadcast::channel(100);
                state.rooms.insert(room_id.to_string(), Room::new(room_id.to_string()));
                state.message_senders.insert(room_id.to_string(), tx);
    }
    
    let room = state.rooms.get_mut(room_id).unwrap();
    
    // Create player
    let player = Player {
        id: player_id.to_string(),
        name: player_name.unwrap_or_else(|| format!("Player {}", player_id)),
        color: None,
    };
    
    // Add player to room
    room.add_player(player)?;
    
    // Create response message
    let response = ServerMessage::RoomJoined {
        room_id: room_id.to_string(),
        player_id: player_id.to_string(),
        players: room.players.clone(),
        game_state: room.game_state.clone(),
    };
    
        // Broadcast to other players in the room
       if let Some(sender) = state.message_senders.get(room_id) {
            if let Err(e) = sender.send(response.clone()) {
                log::warn!("Failed to broadcast RoomJoined message: {:?}", e);
            }
        }
    
    Ok(response)
}

// Send a move
pub fn send_move(room_id: &str, player_id: &str, move_notation: &str) -> Result<ServerMessage, String> {
    let mut state = GAME_STATE.lock().unwrap();
    
    // Check if room exists
    let room = state.rooms.get_mut(room_id).ok_or_else(|| "Room not found".to_string())?;
    
    // Check if player is in the room
    if !room.players.iter().any(|p| p.id == player_id) {
        return Err("Player not in room".to_string());
    }
    
    // Check if game has started
    let game_state = room.game_state.as_mut().ok_or_else(|| "Game not started".to_string())?;
    
    // Apply the move
    game_state.apply_move(move_notation)?;
    
    // Record the move
    room.add_move(player_id.to_string(), move_notation.to_string());
    
    // Create response message
    let response = ServerMessage::MoveMade {
        room_id: room_id.to_string(),
        player_id: player_id.to_string(),
        move_notation: move_notation.to_string(),
        game_state: game_state.clone(),
    };
    
    // Broadcast to all players in the room
    if let Some(sender) = state.message_senders.get(room_id) {
        let _ = sender.send(response.clone());
    }
    
    Ok(response)
}

// Leave a room
pub fn leave_room(room_id: &str, player_id: &str) -> Result<ServerMessage, String> {
    let mut state = GAME_STATE.lock().unwrap();
    
    // Check if room exists
    let room = state.rooms.get_mut(room_id).ok_or_else(|| "Room not found".to_string())?;
    
    // Remove player from room
    if !room.remove_player(player_id) {
        return Err("Player not in room".to_string());
    }
    
    // Create response message
    let response = ServerMessage::PlayerLeft {
        room_id: room_id.to_string(),
        player_id: player_id.to_string(),
    };
    
    // Broadcast to all players in the room
    if let Some(sender) = state.message_senders.get(room_id) {
        let _ = sender.send(response.clone());
    }
    
    // Clean up empty rooms
    if room.players.is_empty() {
        state.rooms.remove(room_id);
        state.message_senders.remove(room_id);
    }
    
    Ok(response)
}

// Get game log
pub fn get_game_log(room_id: &str) -> Result<ServerMessage, String> {
    let state = GAME_STATE.lock().unwrap();
    
    // Check if room exists
    let room = state.rooms.get(room_id).ok_or_else(|| "Room not found".to_string())?;
    
    // Create response message
    let response = ServerMessage::GameLog {
        room_id: room_id.to_string(),
        moves: room.moves.clone(),
    };
    
    Ok(response)
}

// Handle a takeback offer from a player.
// Current behavior: only board state and move history are affected; clocks/time controls are not modified.
pub fn offer_takeback(room_id: &str, player_id: &str) -> Result<ServerMessage, String> {
    let mut state = GAME_STATE.lock().unwrap();

    let room = state
        .rooms
        .get_mut(room_id)
        .ok_or_else(|| "Room not found".to_string())?;

    // Ensure player is in the room
    if !room.players.iter().any(|p| p.id == player_id) {
        return Err("Player not in room".to_string());
    }

    // Require at least one full move (two half-moves) to be able to take back
    if room.moves.len() < 2 {
        return Err("Not enough moves to take back".to_string());
    }

    // Only one pending takeback at a time
    if room.pending_takeback.is_some() {
        return Err("A takeback request is already pending".to_string());
    }

    room.pending_takeback = Some(player_id.to_string());

    let response = ServerMessage::TakebackOffered {
        room_id: room_id.to_string(),
        requester_id: player_id.to_string(),
    };

    if let Some(sender) = state.message_senders.get(room_id) {
        let _ = sender.send(response.clone());
    }

    Ok(response)
}

// Accept a pending takeback request and roll back one full move (two half-moves).
pub fn accept_takeback(room_id: &str, player_id: &str) -> Result<ServerMessage, String> {
    let mut state = GAME_STATE.lock().unwrap();

    let room = state
        .rooms
        .get_mut(room_id)
        .ok_or_else(|| "Room not found".to_string())?;

    // Ensure player is in the room
    if !room.players.iter().any(|p| p.id == player_id) {
        return Err("Player not in room".to_string());
    }

    // There must be a pending takeback request
    let requester_id = match &room.pending_takeback {
        Some(id) => id.clone(),
        None => return Err("No pending takeback request".to_string()),
    };

    // Only the other player (not requester) can accept
    if requester_id == player_id {
        return Err("Requester cannot accept their own takeback".to_string());
    }

    // Need at least one full move (two half-moves) to roll back
    if room.moves.len() < 2 {
        return Err("Not enough moves to take back".to_string());
    }

    // Truncate last two half-moves
    let new_len = room.moves.len() - 2;
    room.moves.truncate(new_len);

    // Rebuild game state from initial position and remaining moves
    let mut game_state = GameState::new_game();
    for mv in &room.moves {
        game_state.apply_move(&mv.move_notation)?;
    }

    room.game_state = Some(game_state.clone());
    room.pending_takeback = None;

    let response = ServerMessage::TakebackAccepted {
        room_id: room_id.to_string(),
        game_state,
        moves: room.moves.clone(),
    };

    if let Some(sender) = state.message_senders.get(room_id) {
        let _ = sender.send(response.clone());
    }

    Ok(response)
}

// Reject a pending takeback request.
pub fn reject_takeback(room_id: &str, player_id: &str) -> Result<ServerMessage, String> {
    let mut state = GAME_STATE.lock().unwrap();

    let room = state
        .rooms
        .get_mut(room_id)
        .ok_or_else(|| "Room not found".to_string())?;

    // Ensure player is in the room
    if !room.players.iter().any(|p| p.id == player_id) {
        return Err("Player not in room".to_string());
    }

    // There must be a pending takeback request
    if room.pending_takeback.is_none() {
        return Err("No pending takeback request".to_string());
    }

    room.pending_takeback = None;

    let response = ServerMessage::TakebackRejected {
        room_id: room_id.to_string(),
        by_player_id: player_id.to_string(),
    };

    if let Some(sender) = state.message_senders.get(room_id) {
        let _ = sender.send(response.clone());
    }

    Ok(response)
}

// Database integration functions
// These are placeholders for future implementation

pub fn save_game_to_db(_room_id: &str) -> Result<(), String> {
    // In a real implementation, this would save the game state to a database
    Ok(())
}

pub fn load_game_from_db(_room_id: &str) -> Result<Room, String> {
    // In a real implementation, this would load the game state from a database
    Err("Not implemented".to_string())
}
