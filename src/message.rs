use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Join { name: String },
    Roll,
    Move { piece_index: usize },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    GameState {
        your_turn: bool,
        // Add actual game state here later
    },
    Error {
        message: String,
    },
    DiceRolled { player_id: Uuid, roll: u8 },
    PieceMoved { player_id: Uuid, piece_index: usize, new_pos: usize },
    TurnSkipped { player_id: Uuid, roll: u8 },
}
