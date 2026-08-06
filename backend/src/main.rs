mod game;
mod state;
mod routes;

use game::game::Game;
use game::player::Player;
use routes::health::get_health;
use shared::message::{ClientMessage, ServerMessage};
use tokio::net::TcpListener;
use tokio::sync::{mpsc,MutexGuard};
use axum::{ Router, routing::get, extract::ws::{WebSocketUpgrade, WebSocket, Message}, response::IntoResponse};
use axum::extract::State;
use state::app::{Lobby, SharedState, Room};
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};
use futures_util::{SinkExt, StreamExt};
use game::action::Action;

#[tokio::main]
async fn main() {
    println!("Play Poker !");

    const BANKROLL: i32 = 200;
    const MAX_PLAYER: usize = 9;

    let mut game = Game::new();

    let state: SharedState = Arc::new(tokio::sync::Mutex::new(Lobby::new(game, BANKROLL, MAX_PLAYER)));

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/health", get(get_health))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<SharedState>) -> impl IntoResponse{
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, lobby: SharedState){
    let (mut sink, mut stream) = socket.split();
    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<ServerMessage>();

    let mut current: Option<(String, u64)> = None;

    loop {
        tokio::select! {
            incoming = stream.next() => {
                let Some(msg) = incoming else { break };
                let text = match msg {
                    Ok(Message::Text(text)) => text,
                    Ok(Message::Ping(data)) => {
                        if sink.send(Message::Pong(data)).await.is_err() { break; }
                        continue;
                    }
                    Ok(Message::Close(_)) | Err(_) => break,
                    Ok(_) => continue,
                };

                let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) else {
                    let _ = out_tx.send(ServerMessage::Error { message: "Invalid message".into() });
                    continue;
                };

                let result = async {
                    let mut lobby = lobby.lock().await;
                    match client_msg {
                        ClientMessage::CreateRoom { name } => {
                            if let Some((old_code, old_id)) = current.clone() {
                                leave_room(&mut lobby, &old_code, old_id).await;
                            }
                            let code = lobby.create_room();
                            let room = lobby.rooms.get(&code).ok_or("Room no longer exists")?;
                            let mut room = room.lock().await;
                            let (conn_id, seat) = room.register(out_tx.clone())?;
                            if let Some(n) = name { room.set_name(conn_id, n); }
                            current = Some((code.clone(), conn_id));
                            let _ = out_tx.send(ServerMessage::RoomCreated { room: code, seat });
                            room.push_state();
                            Ok(())
                        }
                        ClientMessage::JoinRoom { room: code, name } => {
                            if let Some((old_code, old_id)) = current.clone() {
                                leave_room(&mut lobby, &old_code, old_id).await;
                            }
                            let room_ref = lobby.rooms.get(&code).ok_or_else(|| "Room not found".to_string())?;
                            let mut room = room_ref.lock().await;
                            let (conn_id, seat) = room.register(out_tx.clone())?;
                            if let Some(n) = name { room.set_name(conn_id, n); }
                            current = Some((code.clone(), conn_id));
                            let _ = out_tx.send(ServerMessage::Welcome { room: code, seat });
                            room.push_state();
                            Ok(())
                        }
                        ClientMessage::Start => {
                            let (code, conn_id) = current.clone().ok_or("Not in a room")?;
                            let room = lobby.rooms.get(&code).ok_or("Room no longer exists")?;
                            let mut room = room.lock().await;
                            room.start(conn_id)?;
                            room.push_state();
                            Ok(())
                        }
                        ClientMessage::Leave => {
                            let (code, conn_id) = current.clone().ok_or("Not in a room ?")?;
                            leave_room(&mut lobby, &code, conn_id).await;
                            current = None;
                            let _ = out_tx.send(ServerMessage::Leave);
                            Ok(())
                        }
                        ClientMessage::Action { action_type, value } => {
                            let (code, conn_id) = current.clone().ok_or("Not in a room")?;
                            let room_ref = lobby.rooms.get(&code).ok_or("Room no longer exists")?;
                            let mut room = room_ref.lock().await;
                            let action = Action::from_request(&action_type, value)?;
                            room.apply_action(conn_id, action)?;
                            room.push_state();
                            let run_out = room.game.run_out_active();
                            drop(room);
                            if run_out {
                                tokio::spawn(run_out_loop(room_ref.clone()));
                            }
                            Ok(())
                        }
                    }
                }.await;

                if let Err(message) = result {
                    let _ = out_tx.send(ServerMessage::Error { message });
                }
            }
            outgoing = out_rx.recv() => {
                let Some(msg) = outgoing else { break };
                let Ok(text) = serde_json::to_string(&msg) else { continue };
                if sink.send(Message::Text(text.into())).await.is_err() { break; }
            }
        }
    }
    if let Some((code, conn_id)) = current {
        let mut lobby = lobby.lock().await;
        leave_room(&mut lobby, &code, conn_id).await;
    }
}

async fn run_out_loop(room: Arc<tokio::sync::Mutex<Room>>) {
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let mut room = room.lock().await;
        if !room.game.run_out_active() {
            break;
        }
        let finished = room.game.step_run_out();
        room.push_state();
        if finished {
            break;
        }
    }
}

async fn leave_room(lobby: &mut MutexGuard<'_, Lobby>, code: &str, conn_id: u64) {
    let Some(room_ref) = lobby.rooms.get(code).cloned() else { return };
    let mut room = room_ref.lock().await;
    let seat = room.unregister(conn_id);
    let empty = room.connections.is_empty();
    if let Some(seat) = seat {
        room.broadcast(&ServerMessage::PlayerLeft { seat });
    }
    if !empty && room.game.started {
        room.handle_disconnect();
    }
    drop(room);
    if empty {
        lobby.rooms.remove(code);
    }
}