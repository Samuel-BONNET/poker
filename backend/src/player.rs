use crate::card::card::Card;
use crate::hand::hand::Hand;

#[derive(PartialEq, Clone)]
pub struct Player {
    pub id: u64,
    pub name: String,
    pub bankroll: i32,
    pub bet: i32,
    pub total_bet: i32,
    pub hand: Hand,
    pub small_blind: bool,
    pub big_blind: bool,
    pub folded: bool,
    pub all_in: bool,
    pub talked: bool,
    pub active: bool,
}

impl Player {

    pub fn new(id: u64, name: String, bankroll: i32) -> Player {
        Player {
            id,
            name,
            bankroll,
            bet: 0,
            total_bet: 0,
            hand: Hand::new(vec![]),
            small_blind: false,
            big_blind: false,
            folded: false,
            all_in: false,
            talked: false,
            active: true,
        }
    }

    pub fn add_card(&mut self, card: Card){
        self.hand.add_card(card)
    }

    pub fn add_total_bet(&mut self, value: i32){
        self.total_bet += value
    }

    pub fn add_bet(&mut self, value: i32){
        if self.bankroll < self.bet + value{
            self.bet += self.bankroll;
            self.bankroll = 0;
            self.all_in = true;
        }
        else{
            self.bet += value;
            self.bankroll -= value;
        }
    }

    pub fn add_bankroll(&mut self, value: i32){
        self.bankroll += value
    }

    pub fn is_all_in(&self) -> bool{
        self.bankroll == 0
    }

    pub fn clear_hand(&mut self){
        self.hand= Hand::new(vec![])
    }

    pub fn set_folded(&mut self, state: bool){
        self.folded = state
    }

    pub fn set_small_blind(&mut self, state: bool, blind: i32){
        self.small_blind = state;
        self.add_bet(blind)
    }

    pub fn set_big_blind(&mut self, state: bool, blind: i32){
        self.big_blind = state;
        self.add_bet(blind * 2)
    }

    pub fn clear_blind(&mut self){
        self.small_blind = false;
        self.big_blind = false
    }

    pub fn set_talked(&mut self, state: bool) {
        self.talked = state
    }

    pub fn state_active(&mut self, state: bool){
        self.active = state;
    }

    pub fn reset_round_stat(&mut self){
        self.talked = false;
        self.bet = 0
    }

    pub fn restore(&mut self){
        self.reset_round_stat();
        self.clear_hand();
        self.total_bet = 0;
        self.all_in = false;
        self.small_blind = false;
        self.big_blind = false;
        self.folded = false;
    }

}