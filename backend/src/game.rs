use crate::card::deck::Deck;
use crate::player::Player;
use crate::card::card::Card;
use crate::moment::Moment;
use crate::action::Action;
use crate::hand::calculate::HandCalculate;
use crate::hand::hand::Hand;
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
    pub fn add_player(&mut self, player: Player){
        self.players.push(player)
    }

    pub fn remove_player(&mut self, player: Player) -> Option<Player> {
        if let Some(pos) = self.players.iter().position(|p| *p == player){
            Some(self.players.remove(pos))
        }
        else{
            None
        }
    }

    pub fn next_player(&mut self){
        if self.players.len() <= 1{
            return
        }
        let start = self.current_player;
        loop{
            self.current_player = (self.current_player + 1) % self.players.len();
            if !self.players[self.current_player].folded && self.players[self.current_player].active{
                break;
            }
            if self.current_player == start{
                break
            }
        }
    }

    pub fn set_first_player(&mut self){
        self.current_player = self.dealer_index;
        self.next_player()
    }

    pub fn incr_turn(&mut self){
        self.global_turn += 1;
        self.get_blind_level()
    }

    pub fn next_moment(&mut self){
        self.moment = self.moment.next();
    }

    pub fn action_moment(&mut self){
        match self.moment{
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

    pub fn pre_flop(&mut self){
        self.next_button();
        for player in &mut self.players {
            if player.active{
                player.add_card(self.deck.draw().unwrap());
                player.add_card(self.deck.draw().unwrap());
            }
        }
    }

    pub fn flop(&mut self){
        self.deck.draw();
        for _ in 0..3 {
            self.common_card.push(self.deck.draw().unwrap());
        }
    }

    pub fn turn(&mut self){
        self.deck.draw();
        self.common_card.push(self.deck.draw().unwrap())
    }

    pub fn river(&mut self){
        self.deck.draw();
        self.common_card.push(self.deck.draw().unwrap())
    }


    // Run Work

    fn start(&mut self){
        self.clean_cards();
        self.pot = 0;
    }

    pub fn clean_cards(&mut self){
        self.deck.reload();
        self.deck.shuffle();
        self.common_card.clear();
    }

    fn is_end(&self) -> bool {
        let mut count: usize = 0;
        for player in &self.players{
            if !player.folded && player.active{
                count += 1
            }
        }
        count == 1
    }

    fn end(&mut self){
        for player in &mut self.players{
            player.restore();
            if player.active && player.bankroll == 0{
                player.state_active(false)
            }
        }
        self.moment = self.moment.reset();
    }

    fn check_finished(&mut self) -> bool{
        let mut count: usize = 0;
        for player in &self.players {
            if player.active{
                count += 1;
            }
        }
        count == 1
    }

    pub fn all_talked(&self) -> bool{
        for player in &self.players{
            if player.active && !player.talked && !player.folded && !player.is_all_in(){
                return false
            }
        }
        true
    }

    pub fn untalked_all(&mut self){
        for player in &mut self.players{
            player.set_talked(false);
        }
    }

    pub fn bet_gather(&mut self){
        self.max_bet = 0;
        for player in &mut self.players{
            player.add_total_bet(player.bet);
            self.pot += player.bet;
            player.reset_round_stat()
        }
    }

    pub fn distribute_pots(&mut self){
        let results: Vec<(usize, Hand)> = self.players.iter().enumerate()
            .filter(|(_, p)| !p.folded && p.active)
            .map(|(i, p)| {
                let mut c = p.hand.cards.clone();
                c.extend(self.common_card.iter().cloned());
                let mut hc = HandCalculate::new(Hand::new(c));
                (i, hc.best_hands())
            }).collect();

        let mut bets: Vec<(usize, i32)> = results.iter()
            .map(|&(i, _)| (i, self.players[i].total_bet))
            .collect();
        bets.sort_by_key(|&(_, b)| b);

        let dead = self.pot - bets.iter().map(|&(_, b)| b).sum::<i32>();
        let mut prev = 0;

        for (rank, &(_, bet)) in bets.iter().enumerate(){
            if bet <= prev {
                continue
            }
            let eligible: Vec<usize> = bets[rank..].iter().map(|&(i, _)| i).collect();
            let mut amount = (bet - prev) * eligible.len() as i32;
            if rank == bets.len() - 1 { amount += dead; }

            let best = eligible.iter()
                .map(|&i| results.iter().find(|&(j, _)| *j == i).unwrap())
                .reduce(|a, b|
                    if b.1.better_than(&a.1){
                        b
                    }
                    else {
                        a
                    }).unwrap();

            let winners: Vec<&usize> = eligible.iter()
                .filter(|&&i| {
                    let h = &results.iter().find(|&(j, _)| *j == i).unwrap().1;
                    !h.better_than(&best.1) && !best.1.better_than(h)
                }).collect();

            let share = amount / winners.len() as i32;
            for (i, &&w) in winners.iter().enumerate(){
                let extra = if (i as i32) < amount % winners.len() as i32 { 1 } else { 0 };
                self.players[w].add_bankroll(share + extra);
                println!("{} win the {}$ pot !", self.players[w].name, share + extra);
            }
            prev = bet;
        }
    }

    fn choose_button(&mut self){
        let mut rng = rand::thread_rng();
        let size = self.players.len();
        loop{
            let index_random = rng.gen_range(0..size);
            if self.players[index_random].active{
                let mut prev = (index_random + size -1) % size;
                while !self.players[prev].active{
                    prev = (prev + size - 1) % size;
                }
                self.dealer_index = prev;
                break
            }
        }
    }

    fn next_button(&mut self){
        for player in &mut self.players{
            if player.active{
                player.restore()
            }
        }
        let size: usize = self.players.len();
        loop{
            self.dealer_index = (self.dealer_index + 1) % size;
            if self.players[self.dealer_index].active{
                break
            }
        }

        if self.player_remaining().len() == 2{
            self.players[self.dealer_index].set_small_blind(true, self.current_blind);
            for index in 1..size{
                let indx = (self.dealer_index + index) % size;
                if self.players[indx].active{
                    self.players[indx].set_big_blind(true, self.current_blind);
                    self.max_bet = self.current_blind * 2;
                    break
                }
            }
            self.current_player = self.dealer_index;
            return
        }
        for i in 1..size{
            if self.players[(self.dealer_index+i) % size].active{
                self.players[(self.dealer_index+i) % size].set_small_blind(true, self.current_blind);
                break
            }
        }
        for i in 2..size{
            if self.players[(self.dealer_index+i) % size].active && !self.players[(self.dealer_index+i) % size].small_blind{
                self.players[(self.dealer_index+i) % size].set_big_blind(true, self.current_blind);
                self.max_bet = self.current_blind * 2;
                self.current_player = (self.dealer_index+i) % size;
                break
            }
        }
        self.next_player()
    }

    pub fn get_blind_level(&mut self){
        let blind = (1.15_f64).powi(self.global_turn as i32) as i32;
        self.current_blind = blind.clamp(1,50)
    }

    pub fn action_player(&mut self){
        let player = &self.players[self.current_player];
        if !player.folded && player.active{
            println!("\n{} to play (J{})", player.name, player.id);
            println!("Hand: {:?}", player.hand.cards);
            println!("Pot: {} | Max bet: {} | Your bet: {}", self.pot, self.max_bet, player.bet);
            println!("Bankroll: {}", player.bankroll);
            println!("Action : Choose among : fold | check | call | raise <value>");
        }
        else{
            return
        }

        let mut input;
        loop {
            input = String::new();
            io::stdin().read_line(&mut input).unwrap();

            if let Some(action) = Game::parse_action(&input){
                if self.apply_action(action) {
                    break;
                }
            }
            println!("Try again")
        }
        if !self.is_end(){
            self.next_player()
        }
    }

    pub fn parse_action(input: &str) -> Option<Action>{
        let mut parts = input.split_whitespace();
        let command = parts.next();

        match command{
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

    pub fn apply_action(&mut self, action: Action) -> bool{
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

    pub fn check(&mut self) -> bool{
        if self.players[self.current_player].bet == self.max_bet{
            self.players[self.current_player].set_talked(true);
            return true
        }
        println!("Impossible check, pot != max_bet !");
        false
    }

    pub fn call(&mut self) -> bool{
        let gap = self.max_bet - self.players[self.current_player].bet;
        if gap < 0{
            return false
        }
        self.players[self.current_player].add_bet(gap);
        self.players[self.current_player].set_talked(true);
        true
    }

    pub fn raise(&mut self, raise_value: i32) -> bool{
        if raise_value <= 0{
            println!("Not a positive value !");
            return false
        }
        let call_value = self.max_bet - self.players[self.current_player].bet;
        let total_raise = call_value + raise_value;

        if self.players[self.current_player].bankroll >= total_raise{
            self.players[self.current_player].add_bet(total_raise);
            self.max_bet = self.players[self.current_player].bet;
            self.untalked_all();
            self.players[self.current_player].set_talked(true);
            return true
        }
        else{
            println!("You don't have eough to raise");
        }
        false
    }

    pub fn player_remaining(&self) -> Vec<usize>{
        let mut player_list = Vec::new();
        for index in 0..self.players.len(){
            if !self.players[index].folded && self.players[index].active{
                player_list.push(index)
            }
        }
        player_list
    }


    // Gameplay
    pub fn run(&mut self){
        self.choose_button();
        while !self.check_finished(){
            self.incr_turn();
            println!("\nRound {} !", self.global_turn);
            self.start();
            while !self.is_end(){
                self.action_moment();
                println!("\nMoment : {:?}", self.moment);
                if self.moment != Moment::Preflop{
                    println!("Common Cards : {:?}",self.common_card)
                }
                if self.player_remaining().len() == 1{
                    break
                }
                while !self.all_talked() && !self.is_end(){
                    self.action_player()
                }
                self.bet_gather();
                if self.moment == Moment::River{
                    break
                }
                self.next_moment();
                self.set_first_player();
            }
            if self.is_end(){
                let w = self.player_remaining()[0];
                println!("\n{} win {}$ pot !", self.players[w].name, self.pot);
                self.players[w].add_bankroll(self.pot);
            }
            else{
                self.distribute_pots()
            }
            self.end()
        }
    }
}