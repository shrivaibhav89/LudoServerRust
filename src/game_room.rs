use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use tokio::sync::mpsc::UnboundedSender;
use tungstenite::Message;

use crate::player::Player;

#[derive(Debug)]
pub enum GameState {
    Waiting,
    InProgress,
    Finished,
}

pub struct GameRoom {
    pub id: Uuid,
    pub players: HashMap<Uuid, Player>,
    pub state: GameState,
    pub turn_order: Vec<Uuid>,
    pub current_turn_index: usize,
    pub max_players: usize,
    pub created_at: std::time::Instant,
    pub last_dice_roll: Option<u8>,
    pub clients: HashMap<Uuid, UnboundedSender<Message>>,
}

impl GameRoom {
    pub fn new(max_players: usize) -> Self {
        GameRoom {
            id: Uuid::new_v4(),
            players: HashMap::new(),
            state: GameState::Waiting,
            turn_order: Vec::new(),
            current_turn_index: 0,
            max_players,
            created_at: std::time::Instant::now(),
            last_dice_roll: None,
            clients: HashMap::new(),
        }
    }

    pub fn add_player(&mut self, name: &str) -> Option<Uuid> {
        if self.players.len() >= self.max_players {
            return None;
        }

        let player = Player::new(name, self.players.len());
        let id = player.id;
        
        self.players.insert(id, player);
        self.turn_order.push(id);
        if self.players.len() == self.max_players {
            self.state = GameState::InProgress;
        }

        Some(id)
    }

    pub fn current_player(&self) -> Option<&Player> {
        if self.turn_order.is_empty() {
            return None;
        }
        let current_id = self.turn_order[self.current_turn_index];
        self.players.get(&current_id)
    }

    pub fn advance_turn(&mut self) {
        self.current_turn_index = (self.current_turn_index + 1) % self.turn_order.len();
    }

    pub fn is_game_over(&self) -> bool {
        self.players.values().any(|p| p.all_finished())
    }

     pub fn current_player_mut(&mut self) -> Option<&mut Player> {
        if self.turn_order.is_empty() {
            return None;
        }
        let current_id = self.turn_order[self.current_turn_index];
        self.players.get_mut(&current_id)
    }

    /// After a player moves a piece, check if any opponent pieces are on the same position and send them home
    pub fn handle_captures(&mut self, current_player_id: &uuid::Uuid, moved_piece_pos: u8) {
        if moved_piece_pos == 0 || moved_piece_pos == 58 {
            return; // Base or finished, no capture possible
        }

        for (player_id, player) in self.players.iter_mut() {
            if player_id == current_player_id {
                continue; // Skip current player
            }

            for piece_pos in player.pieces.iter_mut() {
                if *piece_pos == moved_piece_pos {
                    println!("💥 Piece capture! Player {}'s piece sent back to base.", player.name);
                    *piece_pos = 0; // Send opponent piece back to base
                }
            }
        }
    }

    pub fn is_current_turn(&self, player_id: &Uuid) -> bool {
            if self.turn_order.is_empty() {
                return false;
            }
         self.turn_order[self.current_turn_index] == *player_id
    }
}