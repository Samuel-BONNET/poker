use crate::game::action::Action;
use crate::Game;
use shared::card::Card;
use shared::message::{GameSnapshot, ServerMessage};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

pub struct Lobby {
    pub rooms: HashMap<String, Arc<Mutex<Room>>>,
    pub template: Game,
    pub next_room_id: u64,
}

pub struct Room {
    pub game: Game,
    pub connections: HashMap<u64, (usize, Sender<ServerMessage>)>,
    pub next_conn_id: u64,
}

pub type SharedState =  Arc<Mutex<Lobby>>;

impl Lobby {
    pub fn new(template: Game) -> Self {
        Lobby { rooms: HashMap::new(), template, next_room_id: 0 }
    }

    pub fn create_room(&mut self) -> String {
        let code = format!("{:04X}", self.next_room_id);
        self.next_room_id += 1;
        let room = Room { game: self.template.clone(), connections: HashMap::new(), next_conn_id: 0 };
        self.rooms.insert(code.clone(), Arc::new(Mutex::new(room)));
        code
    }
}

impl Room {
    pub fn register(&mut self, out: Sender<ServerMessage>) -> Result<(u64, usize), String> {
        let seat = (0..self.game.players.len()).find(|seat| !self.connections.values().any(|(s, _)| s == seat)).ok_or_else(|| "Table full".to_string())?;
        let id = self.next_conn_id;
        self.next_conn_id += 1;
        self.connections.insert(id, (seat, out));
        let player = &mut self.game.players[seat];
        player.set_folded(false);
        if player.bankroll > 0 {
            player.set_active(true);
        }

        Ok((id, seat))
    }

    pub fn unregister(&mut self, id: u64) -> Option<usize> {
        let seat = self.connections.remove(&id).map(|(seat, _)| seat);
        if let Some(seat) = seat {
            self.game.players[seat].active = false;
            self.game.players[seat].folded = true;
        }
        seat
    }

    pub fn set_name(&mut self, id: u64, name: String) -> bool {
        match self.connections.get(&id) {
            Some((seat, _)) => { self.game.players[*seat].name = name; true }
            None => false,
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        if self.connections.len() < 2 {
            return Err("Need at least 2 players to start".to_string());
        }
        self.fold_absent();
        self.game.start_round();
        Ok(())
    }

    fn fold_absent(&mut self) {
        for (seat, player) in self.game.players.iter_mut().enumerate() {
            let connected = self.connections.values().any(|(s, _)| *s == seat);
            if player.active && !connected {
                player.set_active(false);
                player.set_folded(true);
            }
        }
    }

    pub fn apply_action(&mut self, id: u64, action: Action) -> Result<(), String> {
        let seat = self.connections.get(&id).ok_or("Not connected")?.0;
        if seat != self.game.current_player {
            return Err("Not your turn".to_string());
        }
        if !self.game.apply_action(action) {
            return Err("Invalid action".to_string());
        }
        if !self.game.is_end() {
            self.game.next_player();
        }
        self.game.advance();
        Ok(())
    }

    pub fn broadcast(&mut self, msg: &ServerMessage) {
        let mut gone: Vec<u64> = Vec::new();
        for (id, (_, out)) in self.connections.iter() {
            if out.try_send(msg.clone()).is_err() {
                gone.push(*id)
            }
        }
        self.drop_connections(&gone);
    }

    pub fn push_state(&mut self) {
        let snapshot = GameSnapshot::from(&self.game);
        self.broadcast(&ServerMessage::GameState(snapshot));

        let hands: Vec<(u64, Vec<Card>)> = self.connections.iter().map(|(id, (seat, _))| (*id, self.game.player_hand(*seat))).collect();
        let mut gone: Vec<u64> = Vec::new();
        for(id, cards) in hands {
            if let Some((_, out)) = self.connections.get(&id) {
                if out.try_send(ServerMessage::YourHand { cards }).is_err() {
                    gone.push(id);
                }
            }
        }
        self.drop_connections(&gone);
    }

    fn drop_connections(&mut self, ids: &[u64]) {
        let mut seats = Vec::new();
        for id in ids {
            if let Some((seat, _)) = self.connections.remove(id) {
                self.game.players[seat].set_active(false);
                self.game.players[seat].set_folded(true);
                seats.push(seat)
            }
        }
        for seat in seats {
            let msg = ServerMessage::PlayerLeft { seat };
            for (_, (_, out)) in self.connections.iter() {
                let _ = out.try_send(msg.clone());
            }
        }
    }
}