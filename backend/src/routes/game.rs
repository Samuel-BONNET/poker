use axum::Json;
use axum::extract::State;
use serde::Deserialize;
use crate::state::app::SharedState;
use crate::Game;
use crate::game::action::Action;

#[derive(Deserialize)]
pub struct ActionRequest {
    pub action_type: String,
    pub value: Option<i32>,
}

pub async fn get_game(State(state): State<SharedState>) -> Json<Game> {
    let game = state.lock().unwrap();
    Json(game.game.clone())
}

pub async fn post_start(State(state): State<SharedState>) -> Json<Game>{
    let mut game = state.lock().unwrap();
    game.game.start_round();
    Json(game.game.clone())
}

pub async fn post_action(State(state): State<SharedState>, Json(request): Json<ActionRequest>) -> Json<Game>{
    let mut game = state.lock().unwrap();
    let action = match request.action_type.as_str(){
        "fold" => Action::Fold,
        "check" => Action::Check,
        "call" => Action::Call,
        "raise" => Action::Raise(request.value.unwrap_or(0)),
        _ => Action::Fold,
    };

    if game.game.apply_action(action){
        if !game.game.is_end(){
            game.game.next_player();
        }
    };
    game.game.advance();
    Json(game.game.clone())
}