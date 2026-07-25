use crate::card::card::Card;
use crate::card::value::Value;
use crate::hand::rank::Rank;

#[derive(Debug,Clone,PartialEq)]
pub struct Hand{
    pub cards: Vec<Card>,
    pub rank: Rank,
    pub value: Vec<i32>,
}


impl Hand{
    pub fn new(cards: Vec<Card>) -> Self{
        Hand{
            cards,
            rank: Rank::HighCard,
            value: Vec::new()
        }
    }

    pub fn add_card(&mut self, card: Card){
        self.cards.push(card)
    }

    pub fn set_rank(&mut self, new_rank: Rank){
        self.rank = new_rank
    }

    pub fn add_value(&mut self, value: Value){
        self.value.push(value.get_number_value())
    }

    pub fn better_than(&self, best_hand: &Hand) -> bool{
        if self.rank.get_number_rank() != best_hand.rank.get_number_rank(){
            return self.rank.get_number_rank() > best_hand.rank.get_number_rank()
        }
        for i in 0..self.value.len().min(best_hand.value.len()) {
            if self.value[i] != best_hand.value[i] {
                return self.value[i] > best_hand.value[i]
            }
        }
        false
    }
}