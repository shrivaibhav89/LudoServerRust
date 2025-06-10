use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::protocol::Message as WSMessage;
use tokio_tungstenite::WebSocketStream;
use tokio::net::TcpStream;
use futures::stream::SplitSink;
use std::sync::Arc;


use uuid::Uuid;
 use crate::game_room::GameState;

/// Represents a player in the Ludo game
#[derive(Debug, Clone)]
pub struct Player {
    pub id: Uuid,
    pub name: String,
    pub position: usize, // Index in the turn order
    pub is_ready: bool,
    pub pieces: [u8; 4], // 0 = at base, 1-57 = on board, 58 = finished
    pub last_roll: Option<u8>, // NEW}
    //pub sender: Option<Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>>,
}

impl Player {
    pub fn new(name: &str, position: usize) -> Self {
        Player {
            id: Uuid::new_v4(),
            name: name.to_string(),
            position,
            is_ready: false,
            pieces: [0; 4],
            last_roll: None, // INIT
           // sender,
        }
    }

     pub fn set_last_roll(&mut self, value: u8) {
        self.last_roll = Some(value);
    }

    pub fn take_last_roll(&mut self) -> Option<u8> {
        self.last_roll.take()
    }
     
    // Try to move a piece by the dice roll, returns true if moved successfully
    pub fn move_piece(&mut self, piece_index: usize, steps: u8) -> bool {
        if piece_index >= self.pieces.len() {
            return false; // Invalid piece index
        }

        let pos = self.pieces[piece_index];

        // If piece already finished, can't move
        if pos == 58 {
            return false;
        }

        // If piece is at base (0), only move if dice roll is 6
        if pos == 0 {
            if steps == 6 {
                self.pieces[piece_index] = 1; // Move out of base to position 1
                return true;
            } else {
                return false;
            }
        }

        // Move piece forward by steps but max to 58
        let new_pos = pos.saturating_add(steps);

        // Cap at 58 (finished)
        self.pieces[piece_index] = if new_pos > 58 { 58 } else { new_pos };

        true
    }

    // Helper: Check if all pieces finished
    pub fn all_finished(&self) -> bool {
        self.pieces.iter().all(|&p| p == 58)
    }

     pub fn is_piece_finished(&self, pos: i32) -> bool {
        // Adjust 57 to your game's finish position
        pos >= 57
    }

    /// Returns true if the piece can move with the given roll
    pub fn can_move_piece(&self, pos: i32, roll: u8) -> bool {
        if self.is_piece_finished(pos) {
            return false;
        }
        if pos == 0 {
            // At home, can only move out with a 6
            return roll == 6;
        }
        // Can move if it doesn't overshoot the finish
        pos + roll as i32 <= 57
    }


}