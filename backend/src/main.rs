mod game;
mod state;
mod routes;

use game::game::Game;
use game::player::Player;
use routes::game::get_game;
use routes::health::get_health;
use routes::game::post_start;
use routes::game::post_action;
use tokio::net::TcpListener;
use axum::{ Router, routing::{get, post}, extract::ws::{WebSocketUpgrade, WebSocket, Message}, response::IntoResponse};
use state::app::{AppState, SharedState};
use std::sync::{Arc, Mutex};
use tower_http::cors::{CorsLayer, Any};
use futures_util::{SinkExt, StreamExt};

#[tokio::main]
async fn main() {
    println!("Play Poker !");

    const BANKROLL: i32 = 200;

    // setup player
    let player1 = Player::new(1, "Player1".to_string(), BANKROLL);
    let player2 = Player::new(2, "Player2".to_string(), BANKROLL);
    let player3 = Player::new(3, "Player3".to_string(), BANKROLL);
    let player4 = Player::new(4, "Player4".to_string(), BANKROLL);


    // init game + link players
    let mut game = Game::new();
    game.add_player(player1);
    game.add_player(player2);
    game.add_player(player3);
    game.add_player(player4);

    let state: SharedState = Arc::new(Mutex::new(AppState { game }));

    let app = Router::new()
        .route("/game", get(get_game))
        .route("/game/start", post(post_start))
        .route("/game/action", post(post_action))
        .route("/ws", get(ws_handler))
        .route("/health", get(get_health))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse{
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket){
    println!("Connected Client");

    while let Some(msg) = socket.recv().await{
        match msg{
            Ok(Message::Text(text)) => {
                println!("Received : {}", text);

                socket.send(Message::Text("Received Message".into())).await.unwrap();
            }

            Ok(Message::Close(_)) => {
                println!("Disconnected Client");
                break;
            }
            _ => {}
        }
    }
}
