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
use state::app::{Lobby, SharedState};
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};
use futures_util::{SinkExt, StreamExt};
use game::action::Action;

#[tokio::main]
async fn main() {
    println!("Play Poker !");

    const BANKROLL: i32 = 200;

    let player1 = Player::new(1, "Player1".to_string(), BANKROLL);
    let player2 = Player::new(2, "Player2".to_string(), BANKROLL);
    let player3 = Player::new(3, "Player3".to_string(), BANKROLL);
    let player4 = Player::new(4, "Player4".to_string(), BANKROLL);


    let mut game = Game::new();
    game.add_player(player1);
    game.add_player(player2);
    game.add_player(player3);
    game.add_player(player4);

    let state: SharedState = Arc::new(tokio::sync::Mutex::new(Lobby::new(game)));

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
    let (out_tx, mut out_rx) = mpsc::channel::<ServerMessage>(64);

    let send_task = tokio::spawn(async move {
        while let Some(msg) = out_rx.recv().await {
            let Ok(text) = serde_json::to_string(&msg) else { continue };
            if sink.send(Message::Text(text.into())).await.is_err() {
                break;
            }
        }
    });

    let mut current: Option<(String, u64)> = None;

    while let Some(msg) = stream.next().await {
        let text = match msg {
            Ok(Message::Text(text)) => text,
            Ok(Message::Close(_)) => break,
            Ok(_) => continue,
            Err(_) => break,
         };

        let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) else {
            let _ = out_tx.try_send(ServerMessage::Error { message: "Invalid message".into() });
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
                    let mut room = lobby.rooms.get(&code).unwrap().lock().await;
                    let (conn_id, seat) = room.register(out_tx.clone())?;
                    if let Some(n) = name { room.set_name(conn_id, n); }
                    current = Some((code.clone(), conn_id));
                    let _ = out_tx.try_send(ServerMessage::RoomCreated { room: code, seat });
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
                    let _ = out_tx.try_send(ServerMessage::Welcome { room: code, seat });
                    room.push_state();
                    Ok(())
                }
                ClientMessage::Start => {
                    let (code, _) = current.clone().ok_or("Not in a room")?;
                    let mut room = lobby.rooms.get(&code).unwrap().lock().await;
                    room.start()?;
                    room.push_state();
                    Ok(())
                }
                ClientMessage::Action { action_type, value } => {
                    let (code, conn_id) = current.clone().ok_or("Not in a room")?;
                    let mut room = lobby.rooms.get(&code).unwrap().lock().await;
                    room.apply_action(conn_id, Action::from_request(&action_type, value))?;
                    room.push_state();
                    Ok(())
                }
            }
        }.await;

        if let Err(message) = result {
            let _ = out_tx.try_send(ServerMessage::Error { message });
        }
    }

    if let Some((code, conn_id)) = current {
        let mut lobby = lobby.lock().await;
        leave_room(&mut lobby, &code, conn_id).await;
    }
    send_task.abort();
}

async fn leave_room(lobby: &mut MutexGuard<'_, Lobby>, code: &str, conn_id: u64) {
    let Some(room_ref) = lobby.rooms.get(code).cloned() else { return };
    let mut room = room_ref.lock().await;
    let seat = room.unregister(conn_id);
    let empty = room.connections.is_empty();
    if let Some(seat) = seat {
        room.broadcast(&ServerMessage::PlayerLeft { seat });
    }
    drop(room);
    if empty {
        lobby.rooms.remove(code);
    }
}