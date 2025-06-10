use futures::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::accept_async;
use tungstenite::protocol::Message;

use crate::game_logic;
use crate::message::{ClientMessage, ServerMessage};
use serde_json;

use crate::game_room::GameRoom;
use crate::player::Player;
use rand;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

type SharedRooms = Arc<Mutex<HashMap<String, GameRoom>>>;

pub async fn start_websocket_server() {
    let listener = TcpListener::bind("127.0.0.1:9001")
        .await
        .expect("Failed to bind");

    let rooms: SharedRooms = Arc::new(Mutex::new(HashMap::new()));

    println!("🌐 WebSocket running on ws://127.0.0.1:9001");

    while let Ok((stream, _)) = listener.accept().await {
        let rooms = Arc::clone(&rooms);
        tokio::spawn(handle_client(stream, rooms));
    }
}

async fn handle_client(stream: TcpStream, rooms: SharedRooms) {
    let ws_stream = accept_async(stream).await.unwrap();
    println!("🟢 New WebSocket connection accepted");
    let (mut write, mut read) = ws_stream.split();

    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    // Spawn a task to forward messages from rx to the WebSocket
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let _ = write.send(msg).await;
        }
    });

    let mut player_id: Option<Uuid> = None;

    while let Some(Ok(msg)) = read.next().await {
        println!("⬅️ Received message: {:?}", msg);
        if let Ok(text) = msg.to_text() {
            match serde_json::from_str::<ClientMessage>(text) {
                Ok(ClientMessage::Join { name }) => {
                    let mut rooms = rooms.lock().await;
                    println!("✅ {} joined room as {:?}", name, player_id);
                    let room = rooms
                        .entry("default".to_string())
                        .or_insert_with(|| GameRoom::new(2));

                    // or however you track position
                    let position = room.players.len();
                    let player = Player::new(&name, position);
                    if let Some(assigned_id) = room.add_player(&name) {
                        println!("✅ {} joined room", name);
                        player_id = Some(assigned_id);
                        room.clients.insert(assigned_id, tx.clone());
                        println!(
                            "📢 Sending turn info: is it your turn? {} (Player ID: {})",
                            room.is_current_turn(&assigned_id),
                            assigned_id
                        );

                        let response = ServerMessage::GameState {
                            your_turn: room.is_current_turn(&assigned_id),
                        };

                        let json = serde_json::to_string(&response).unwrap();
                        let _ = tx.send(Message::text(json));
                    } else {
                        let error = ServerMessage::Error {
                            message: "Room is full.".into(),
                        };
                        let json = serde_json::to_string(&error).unwrap();
                        let _ = tx.send(Message::text(json));
                    }
                }
                Ok(ClientMessage::Roll) => {
                    let mut rooms = rooms.lock().await;
                    let pid = player_id.unwrap();
                    if let Some(room) = rooms.get_mut("default") {
                        if room.is_current_turn(&pid) {
                            let roll = rand::random::<u8>() % 6 + 1;
                            room.last_dice_roll = Some(roll);
                            println!("🎲 Player {:?} rolled a {}", pid, roll);
                            let player = room.players.get_mut(&pid).unwrap();
                            player.last_roll = Some(roll);
                            let can_move = player.pieces.iter().any(|&pos| {
                                // Replace this logic with your own game rules
                                // Example: piece is not finished and can move with this roll
                                !player.is_piece_finished(pos as i32)
                                    && player.can_move_piece(pos as i32, roll)
                            });

                            if !can_move {
                                // No move possible, advance turn
                                println!(
                                    "⏭️ No move possible for player {:?}, skipping turn.",
                                    pid
                                );
                                room.advance_turn();

                                // Notify all clients (optional)
                                let skip_msg = ServerMessage::TurnSkipped {
                                    player_id: pid,
                                    roll,
                                };
                                let json = serde_json::to_string(&skip_msg).unwrap();
                                for sender in room.clients.values() {
                                    let _ = sender.send(Message::text(json.clone()));
                                }

                                // Optionally, send GameState to the player
                                let response = ServerMessage::GameState { your_turn: false };
                                let json = serde_json::to_string(&response).unwrap();
                                let _ = tx.send(Message::text(json));
                                continue;
                            }
                            let mut still_your_turn = false;
                            if roll == 6 {
                                println!("🔥 Rolled a 6! Player gets another turn.");
                                still_your_turn = true;
                            } else {
                                //room.advance_turn();
                            }
                            let response = ServerMessage::GameState {
                                your_turn: still_your_turn, // still player's turn
                                                            // later: include board state, dice, etc.
                            };
                            let json = serde_json::to_string(&response).unwrap();
                            let _ = tx.send(Message::text(json));

                            let roll_msg = ServerMessage::DiceRolled {
                                player_id: pid,
                                roll,
                            };
                            let json = serde_json::to_string(&roll_msg).unwrap();
                            for sender in room.clients.values() {
                                let _ = sender.send(Message::text(json.clone()));
                            }
                        } else {
                            let error = ServerMessage::Error {
                                message: "It's not your turn.".into(),
                            };
                            let json = serde_json::to_string(&error).unwrap();
                            let _ = tx.send(Message::text(json));
                        }
                    }
                }

                Ok(ClientMessage::Move { piece_index }) => {
                    let mut rooms = rooms.lock().await;
                    let pid = player_id.unwrap();
                    if let Some(room) = rooms.get_mut("default") {
                        if !room.is_current_turn(&pid) {
                            let error = ServerMessage::Error {
                                message: "Not your turn.".into(),
                            };
                            let json = serde_json::to_string(&error).unwrap();
                            let _ = tx.send(Message::text(json));
                            continue;
                        }

                        // Safely extract roll + update player first
                        let (roll, new_pos);
                        {
                            let player = room.players.get_mut(&pid).unwrap();

                            if piece_index >= player.pieces.len() {
                                let error = ServerMessage::Error {
                                    message: "Invalid piece index.".into(),
                                };
                                let json = serde_json::to_string(&error).unwrap();
                                let _ = tx.send(Message::text(json));
                                continue;
                            }
                            let player = room.players.get_mut(&pid).unwrap();
                            if let Some(r) = player.take_last_roll() {
                                roll = r;
                                player.pieces[piece_index] += roll;
                                new_pos = player.pieces[piece_index];
                                println!(
                                    "🚚 Player {} moved piece {} by {}",
                                    player.name, piece_index, roll
                                );

                                let moved_global = game_logic::get_global_board_index(
                                    player.position,
                                    player.pieces[piece_index].into(),
                                );
                                for (other_id, other_player) in room.players.iter_mut() {
                                    if *other_id == pid {
                                        continue;
                                    }
                                    for (i, &other_pos) in other_player.pieces.iter().enumerate() {
                                        let other_global =  game_logic::get_global_board_index(
                                            other_player.position,
                                            other_pos.into(),
                                        );
                                        if moved_global.is_some() && moved_global == other_global {
                                            // Capture logic: send other_player.pieces[i] home
                                           // other_player.pieces[i] = 0;
                                            // Optionally notify all clients about the capture
                                        }
                                    }
                                }

                                let move_msg = ServerMessage::PieceMoved {
                                    player_id: pid,
                                    piece_index,
                                    new_pos: new_pos.into(),
                                };
                                let json = serde_json::to_string(&move_msg).unwrap();
                                for sender in room.clients.values() {
                                    let _ = sender.send(Message::text(json.clone()));
                                }
                            } else {
                                let error = ServerMessage::Error {
                                    message: "You must roll the dice before moving.".into(),
                                };
                                let json = serde_json::to_string(&error).unwrap();
                                let _ = tx.send(Message::text(json));
                                continue;
                            }
                        }

                        // Now it's safe to mutably access `room` again
                        room.handle_captures(&pid, new_pos);

                        // Check game over and advance
                        let player = &room.players[&pid];
                        if player.all_finished() {
                            room.state = crate::game_room::GameState::Finished;
                            println!("🎉 {} has finished the game!", player.name);
                        }

                        // Only advance turn if the last roll was NOT a 6
                        if roll != 6 {
                            room.advance_turn();
                        }

                        let response = ServerMessage::GameState {
                            your_turn: room.is_current_turn(&pid),
                        };
                        let json = serde_json::to_string(&response).unwrap();
                        let _ = tx.send(Message::text(json));
                    }
                }

                _ => {
                    // TODO: handle Roll, Move
                }
            }
        }
    }

    println!("🔌 Player {} disconnected", player_id.unwrap());
}
