use crate::game::action::Action;
use crate::game::game::Game;
use crate::game::player::Player;
use shared::card::Card;
use shared::message::{GameSnapshot, ServerMessage};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex;

pub struct Lobby {
    pub rooms: HashMap<String, Arc<Mutex<Room>>>,
    pub template: Game,
    pub next_room_id: u64,
    pub bankroll: i32,
    pub max_players: usize,
}

pub struct Room {
    pub leader: u64,
    pub game: Game,
    pub connections: HashMap<u64, (usize, UnboundedSender<ServerMessage>)>,
    pub next_conn_id: u64,
    pub bankroll: i32,
    pub max_players: usize,
    pub next_player_id: u64,
}

pub type SharedState =  Arc<Mutex<Lobby>>;

impl Lobby {
    pub fn new(template: Game, bankroll: i32, max_players: usize) -> Self {
        Lobby { rooms: HashMap::new(), template, next_room_id: 0, bankroll, max_players }
    }

    pub fn create_room(&mut self) -> String {
        let code = format!("{:06X}", self.next_room_id);
        self.next_room_id += 1;
        let mut room = Room {
            game: self.template.clone(),
            connections: HashMap::new(),
            next_conn_id: 0,
            leader: 0,
            bankroll: self.bankroll,
            max_players: self.max_players,
            next_player_id: 1,
        };
        self.rooms.insert(code.clone(), Arc::new(Mutex::new(room)));
        code
    }
}

impl Room {
    pub fn register(&mut self, out: UnboundedSender<ServerMessage>) -> Result<(u64, usize), String> {
        if self.game.started { return Err("Game already started".to_string()); }

        let seat = match (0..self.game.players.len()).find(|seat| !self.connections.values().any(|(s, _)| s == seat)) {
            Some(seat) => {
                let player = &mut self.game.players[seat];
                player.set_folded(false);
                if player.bankroll > 0 {
                    player.set_active(true);
                }
                seat
            }
            None => {
                if self.game.players.len() >= self.max_players {
                    return Err("Table full".to_string());
                }
                let pid = self.next_player_id;
                self.next_player_id += 1;
                let seat = self.game.players.len();
                self.game.add_player(Player::new(pid, "Player".to_string(), self.bankroll));
                seat
            }
        };
        let id = self.next_conn_id;
        self.next_conn_id += 1;
        if self.connections.is_empty() {
            self.leader = id;
        }
        self.connections.insert(id, (seat, out));
        Ok((id, seat))
    }

    pub fn unregister(&mut self, id: u64) -> Option<usize> {
        let seat = self.connections.remove(&id).map(|(seat, _)| seat);
        if let Some(seat) = seat {
            self.game.players[seat].active = false;
            self.game.players[seat].folded = true;
        }
        self.reelect_leader();
        seat
    }

    fn reelect_leader(&mut self) {
        if self.leader != 0 && self.connections.contains_key(&self.leader) { return; }
        self.leader = self.connections.keys().min().copied().unwrap_or(0);
    }

    pub fn set_name(&mut self, id: u64, name: String) -> bool {
        match self.connections.get(&id) {
            Some((seat, _)) => { self.game.players[*seat].name = name; true }
            None => false,
        }
    }

    pub fn start(&mut self, requester: u64) -> Result<(), String> {
        if requester != self.leader {
            return Err("Only the host can start".to_string())
        }
        if self.game.started {
            return Err("Game already started".to_string())
        }
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
        if self.game.run_out_active() {
            return Err("Showdown in progress".to_string());
        }
        if !self.game.started { return Err("Game not started".to_string()); }
        let seat = self.connections.get(&id).ok_or("Not connected")?.0;
        if seat != self.game.current_player {
            return Err("Not your turn".to_string());
        }
        if !self.game.apply_action(action) {
            return Err("Invalid action".to_string());
        }
        self.game.next_player();
        self.game.advance();
        Ok(())
    }

    pub fn broadcast(&mut self, msg: &ServerMessage) {
        let mut gone: Vec<u64> = Vec::new();
        for (id, (_, out)) in self.connections.iter() {
            if out.send(msg.clone()).is_err() {
                gone.push(*id)
            }
        }
        self.drop_connections(&gone);
    }

    pub fn push_state(&mut self) {
        let snapshot = {
            let mut s = GameSnapshot::from(&self.game);
            s.leader_seat = self.connections.get(&self.leader).map(|(seat, _)| *seat);
            s
        };
        self.broadcast(&ServerMessage::GameState(snapshot));

        let hands: Vec<(u64, Vec<Card>)> = self.connections.iter().map(|(id, (seat, _))| (*id, self.game.player_hand(*seat))).collect();
        let mut gone: Vec<u64> = Vec::new();
        for(id, cards) in hands {
            if let Some((_, out)) = self.connections.get(&id) {
                if out.send(ServerMessage::YourHand { cards }).is_err() {
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
                let _ = out.send(msg.clone());
            }
        }
        self.reelect_leader()
    }

    pub fn handle_disconnect(&mut self) {
        if !self.game.started || self.game.run_out_active() {
            self.push_state();
            return;
        }
        let cp = self.game.current_player;
        let p = &self.game.players[cp];
        if p.folded || !p.active || p.is_all_in() {
            self.game.next_player();
            self.game.advance();
        }
        self.push_state();
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn make_room(max_players: usize) -> Arc<Mutex<Room>> {
        let mut lobby = Lobby::new(Game::new(), 200, max_players);
        let code = lobby.create_room();
        lobby.rooms.remove(&code).unwrap()
    }

    fn register(room: &Arc<Mutex<Room>>) -> Result<(u64, usize), String> {
        let (out, _) = tokio::sync::mpsc::unbounded_channel();
        room.try_lock().unwrap().register(out)
    }

    #[test]
    fn start_requires_at_least_two_players() {
        let room = make_room(9);
        register(&room).unwrap();
        assert!(room.try_lock().unwrap().start(0).is_err());
    }

    #[test]
    fn starts_with_exactly_the_joined_players() {
        let room = make_room(9);
        let (leader, _) = register(&room).unwrap();
        register(&room).unwrap();
        let mut guard = room.try_lock().unwrap();
        guard.start(leader).unwrap();
        assert_eq!(guard.game.players.len(), 2);
        assert_eq!(guard.game.players.iter().filter(|p| p.active).count(), 2);
    }

    #[test]
    fn seats_are_added_dynamically_until_max() {
        let room = make_room(5);
        for i in 0..5 {
            assert!(register(&room).is_ok(), "player {i} should join");
        }
        assert!(register(&room).is_err());
    }

    #[test]
    fn all_in_does_not_soft_lock() {
        let room = make_room(3);
        let (id0, _) = register(&room).unwrap();
        let (id1, _) = register(&room).unwrap();
        let (id2, _) = register(&room).unwrap();
        let ids = [id0, id1, id2];
        let mut guard = room.try_lock().unwrap();
        guard.start(id0).unwrap();

        let first = guard.game.current_player;
        guard.apply_action(ids[first], Action::AllIn).unwrap();
        let second = guard.game.current_player;
        guard.apply_action(ids[second], Action::AllIn).unwrap();

        let chip_holder = guard.game.players.iter().enumerate()
            .filter(|(_, p)| p.active && !p.folded && p.bankroll > 0)
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        assert_eq!(chip_holder.len(), 1);
        assert_eq!(guard.game.current_player, chip_holder[0]);

        guard.apply_action(ids[chip_holder[0]], Action::AllIn).unwrap();
        assert!(guard.game.run_out_active());

        let mut finished = false;
        for _ in 0..5 {
            finished = guard.game.step_run_out();
            if finished { break; }
        }
        assert!(finished);
        assert!(!guard.game.run_out);
    }
}