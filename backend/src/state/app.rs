use crate::Game;
use std::sync::{Arc,Mutex};

pub struct AppState {
    pub game: Game,
}

pub type SharedState =  Arc<Mutex<AppState>>;