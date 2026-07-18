use crate::card::deck::Deck;
use crate::player::Player;
use crate::card::card::Card;
use crate::moment::Moment;
use crate::action::Action;
use std::io;
use rand::Rng;

pub struct Game {
    pub players: Vec<Player>,
    pub current_player: usize,
    pub dealer_index: usize,
    pub global_turn: usize,
    pub pot: i32,
    pub max_bet: i32,
    pub current_blind: i32,
    pub deck: Deck,
    pub common_card: Vec<Card>,
    pub moment: Moment,
}

impl Game {
    pub fn new() -> Game {
        Game {
            players: Vec::new(),
            current_player: 0,
            dealer_index: 0,
            global_turn: 0,
            pot: 0,
            max_bet: 0,
            current_blind: 1,
            deck: Deck::new(),
            common_card: Vec::new(),
            moment: Moment::Preflop,
        }
    }

    // Game Data
    pub fn add_player(&mut self, player: Player) {
        self.players.push(player)
    }

    pub fn remove_player(&mut self, player: Player) -> Option<Player> {
        if let Some(pos) = self.players.iter().position(|p| *p == player) {
            Some(self.players.remove(pos))
        } else {
            None
        }
    }

    pub fn next_player(&mut self) {
        if self.players.len() <= 1 {
            return
        }
        let start = self.current_player;
        loop {
            self.current_player = (self.current_player + 1) % self.players.len();
            if !self.players[self.current_player].folded && self.players[self.current_player].active {
                break;
            }
            if self.current_player == start {
                break
            }
        }
    }

    pub fn set_first_player(&mut self) {
        self.current_player = self.dealer_index;
        self.next_player()
    }

    pub fn incr_turn(&mut self) {
        self.global_turn += 1
    }

    pub fn next_moment(&mut self) {
        self.moment = self.moment.next();
    }

    pub fn action_moment(&mut self) {
        match self.moment {
            Moment::Preflop => {
                self.pre_flop()
            },
            Moment::Flop => {
                self.flop()
            },
            Moment::Turn => {
                self.turn()
            },
            Moment::River => {
                self.river()
            }
        }
    }

    pub fn pre_flop(&mut self) {
        self.next_button();
        for player in &mut self.players {
            if player.active {
                player.add_card(self.deck.draw().unwrap());
                player.add_card(self.deck.draw().unwrap());
            }
        }
    }

    pub fn flop(&mut self) {
        self.deck.draw();
        for _ in 0..3 {
            self.common_card.push(self.deck.draw().unwrap());
        }
    }

    pub fn turn(&mut self) {
        self.deck.draw();
        self.common_card.push(self.deck.draw().unwrap())
    }

    pub fn river(&mut self) {
        self.deck.draw();
        self.common_card.push(self.deck.draw().unwrap())
    }

    pub fn action_player(&mut self) {
        let player = &self.players[self.current_player];
        if !player.folded && player.active {
            println!("\n{} to play (J{})", player.name, player.id);
            println!("Hand: {:?}", player.hand.cards);
            println!("Pot: {} | Max bet: {} | Your bet: {}", self.pot, self.max_bet, player.bet);
            println!("Bankroll: {}", player.bankroll);
            println!("Action : Choose among : fold | check | call | raise <value>");
        } else {
            return
        }

        let mut input;
        loop {
            input = String::new();
            io::stdin().read_line(&mut input).unwrap();

            if let Some(action) = Game::parse_action(&input) {
                if self.apply_action(action) {
                    break;
                }
            }
            println!("Try again")
        }
        if !self.is_end() {
            self.next_player()
        }
    }

    pub fn parse_action(input: &str) -> Option<Action> {
        let mut parts = input.split_whitespace();
        let command = parts.next();

        match command {
            Some("fold") => Some(Action::Fold),
            Some("check") => Some(Action::Check),
            Some("call") => Some(Action::Call),
            Some("raise") => Some(Action::Raise(parts.next()?.parse::<i32>().ok()?)),
            _ => {
                println!("Input error");
                None
            }
        }
    }

    pub fn apply_action(&mut self, action: Action) -> bool {
        match action {
            Action::Fold => {
                self.players[self.current_player].set_folded(true);
                true
            }

            Action::Check => {
                self.check()
            }

            Action::Call => {
                self.call()
            }

            Action::Raise(value) => {
                self.raise(value)
            }
        }
    }

    pub fn check(&mut self) -> bool {
        if self.players[self.current_player].bet == self.max_bet {
            self.players[self.current_player].set_talked(true);
            return true
        }
        println!("Impossible check, pot != max_bet !");
        false
    }

    pub fn call(&mut self) -> bool {
        let gap = self.max_bet - self.players[self.current_player].bet;
        if gap < 0 {
            return false
        }
        self.players[self.current_player].add_bet(gap);
        self.players[self.current_player].set_talked(true);
        true
    }

    pub fn raise(&mut self, raise_value: i32) -> bool {
        if raise_value <= 0 {
            println!("Not a positive value !");
            return false
        }
        let call_value = self.max_bet - self.players[self.current_player].bet;
        let total_raise = call_value + raise_value;

        if self.players[self.current_player].bankroll >= total_raise {
            self.players[self.current_player].add_bet(total_raise);
            self.max_bet = self.players[self.current_player].bet;
            self.untalked_all();
            self.players[self.current_player].set_talked(true);
            return true
        } else {
            println!("You don't have eough to raise");
        }
        false
    }

    pub fn player_remaining(&self) -> Vec<usize> {
        let mut player_list = Vec::new();
        for index in 0..self.players.len() {
            if !self.players[index].folded && self.players[index].active {
                player_list.push(index)
            }
        }
        player_list
    }
}