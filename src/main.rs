use mongodb::bson::doc;
use mongodb::{Client, options::ClientOptions};
use serde::{Deserialize, Serialize};
mod db;

mod player;
mod game_room;
mod game_logic;
mod message;

mod server;

use game_logic::roll_dice;
use game_room::GameRoom;

// #[derive(Debug, Serialize, Deserialize)]
// struct Player {
//     username: String,
//     score: i32,
// }

// #[tokio::main]
// async fn main() {
    
//     // db::db::insert_player().await.unwrap_or_else(|err| {
//     //     eprintln!("Error inserting player: {}", err);
//     // });


//     // db::db::get_players().await.unwrap_or_else(|err| {
//     //     eprintln!("Error getting players: {}", err);
//     // });

//     // db::db::update_player_score("vaibhav_simple new module", 200000).await.unwrap_or_else(|err| {
//     //     eprintln!("Error updating player score: {}", err);
//     // });
// }
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use futures_util::{StreamExt, SinkExt};
use tokio::spawn;

// //mod models;

// use axum::{
//     routing::post,
//     Json, Router,
//     extract::State,
// };
// use models::User;


#[tokio::main]
async fn main() {
    server::start_websocket_server().await;
}

// fn main() {
//   println!("🎲 Starting Ludo Game Simulation (2 Players + Dice Rolls)...");

//     println!("🎲 Starting Ludo Simulation (2 Players + Captures + Extra Turns)...");

//     let mut room = GameRoom::new(2);

//     let names = ["Alice", "Bob"];
//     for name in names.iter() {
//         let player_id = room.add_player(name);
//         match player_id {
//             Some(id) => println!("✅ Added player: {} ({})", name, id),
//             None => println!("❌ Failed to add player: {}", name),
//         }
//     }

//     println!("\n🕹 Game State: {:?}", room.state);

//     let mut turn = 0;

//     while turn < 20 {
//         // Get current player id before any mutable borrow
//         let current_id = {
//             let idx = room.current_turn_index;
//             room.turn_order[idx]
//         };

//         // Scope 1: Mutable borrow for player to do move
//         let (dice_value, moved, moved_piece_pos, player_won) = {
//             let current_player = room.current_player_mut().unwrap();

//             println!("\n🔁 Turn {} - {}'s turn", turn + 1, current_player.name);

//             let dice_value = roll_dice();
//             println!("🎲 Rolled: {}", dice_value);

//             let mut moved = false;
//             let mut moved_piece_pos = 0;

//             for piece_index in 0..4 {
//                 if current_player.move_piece(piece_index, dice_value) {
//                     moved_piece_pos = current_player.pieces[piece_index];
//                     println!(
//                         "🚀 Moved piece {} to position {}",
//                         piece_index + 1,
//                         moved_piece_pos
//                     );
//                     moved = true;
//                     break;
//                 }
//             }

//             if !moved {
//                 println!("⏭️ No pieces moved this turn");
//             }

//             let player_won = current_player.all_finished();

//             (dice_value, moved, moved_piece_pos, player_won)
//         };

//         // Scope 2: Mutable borrow of room to handle captures
//         if moved {
//             room.handle_captures(&current_id, moved_piece_pos);
//         }

//         // Check win condition outside player mutable borrow scope
//         if player_won {
//             let player = room.players.get(&current_id).unwrap();
//             println!("🎉 {} has won the game!", player.name);
//             break;
//         }

//         // Advance turn or give extra turn if dice was 6
//         if dice_value != 6 {
//             room.advance_turn();
//             turn += 1;
//         } else {
//             println!("🎉 {} rolled a 6 and gets an extra turn!", room.players.get(&current_id).unwrap().name);
//             // Do not advance turn or increment turn count
//         }
//     }

//     println!("\n✅ Simulation Complete.");
// }


// #[tokio::main]
// async fn main() {

//     let listener = TcpListener::bind("0.0.0.0:9001").await.expect("Failed to bind");

//     println!("🚀 WebSocket server listening on port 9001");

//     while let Ok((stream, _)) = listener.accept().await {
//         spawn(async move {
//             let ws_stream = accept_async(stream).await.expect("WebSocket handshake failed");
//             println!("🟢 New WebSocket connection");

//             let (mut write, mut read) = ws_stream.split();

//             // Send a welcome message to client
//             let welcome_msg = tokio_tungstenite::tungstenite::Message::Text("Hello from Rust server".into());
//             if let Err(e) = write.send(welcome_msg).await {
//                 eprintln!("❌ Send error: {}", e);
//             }

//             // Echo loop
//             while let Some(msg) = read.next().await {
//                 match msg {
//                     Ok(msg) => {
//                         println!("⬅️ Received: {:?}", msg);
//                         if let Err(e) = write.send(msg).await {
//                             eprintln!("❌ Echo send error: {}", e);
//                             break;
//                         }
//                     }
//                     Err(e) => {
//                         eprintln!("❌ Receive error: {}", e);
//                         break;
//                     }
//                 }
//             }

//             println!("🔴 Client disconnected");
//         });
//     }
// }
