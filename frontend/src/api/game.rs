use gloo_net::http::Request;
use serde::Deserialize;
use shared::color::Color;
use shared::value::Value;



#[derive(Deserialize, Debug, Clone)]
pub struct GameState{
    pub pot: i32,
    pub max_bet: i32,
    pub current_player: usize,
    pub dealer_index: usize,
    pub moment: String,
    pub common_card: Vec<CardState>,
    pub players: Vec<PlayerState>,
}


#[derive(Deserialize, Debug, Clone)]
pub struct PlayerState{
    pub id: u64,
    pub name: String,
    pub bankroll: i32,
    pub bet: i32,
    pub hand: HandState,
    pub folded: bool,
    pub all_in: bool,
    pub small_blind: bool,
    pub big_blind: bool,
    pub talked: bool,
    pub active: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct HandState{
    pub cards: Vec<CardState>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CardState{
    pub color: Color,
    pub value: Value,
}

pub async fn fetch_game() -> Result<GameState, ()>{
    let resp = Request::get("http://127.0.0.1:3000/game")
        .send().await.map_err(|_| ())?;
    resp.json::<GameState>().await.map_err(|_| ())
}

pub async fn start_game() -> Result<GameState, ()>{
    let resp = Request::post("http://127.0.0.1:3000/game/start")
        .send().await.map_err(|_| ())?;
    resp.json::<GameState>().await.map_err(|_| ())
}

pub async fn send_action(action_type: &str, value: Option<i32>) -> Result<GameState, ()>{
    let body = serde_json::json!({
        "action_type": action_type,
        "value": value,
    });
    let resp = Request::post("http://127.0.0.1:3000/game/action")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&body).unwrap())
        .map_err(|_| ())?
        .send().await.map_err(|_| ())?;
    resp.json::<GameState>().await.map_err(|_| ())
}