use serde::{Serialize, Deserialize};
use crate::card::Card;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    CreateRoom { name: Option<String> },
    JoinRoom { room: String, name: Option<String> },
    Start,
    Action { action_type: String, value: Option<i32> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    RoomCreated { room: String, seat: usize },
    Welcome { room: String, seat: usize },
    GameState(GameSnapshot),
    YourHand { cards: Vec<Card> },
    PlayerLeft { seat: usize },
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSnapshot {
    pub pot: i32,
    pub max_bet: i32,
    pub current_player: usize,
    pub dealer_index: usize,
    pub moment: String,
    pub common_card: Vec<Card>,
    pub players: Vec<PlayerSnapshot>,
    pub started: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSnapshot {
    pub id: u64,
    pub name: String,
    pub bankroll: i32,
    pub bet: i32,
    pub hand: Vec<Card>,
    pub folded: bool,
    pub all_in: bool,
    pub small_blind: bool,
    pub big_blind: bool,
    pub talked: bool,
    pub active: bool,
}