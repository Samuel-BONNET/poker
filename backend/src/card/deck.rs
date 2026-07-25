use super::color::Color;
use super::value::Value;
use super::card::Card;
use rand::seq::SliceRandom;

#[derive(Debug)]
pub struct Deck {
    cards: Vec<Card>
}

impl Deck {

    pub fn new() -> Self {
        Deck {
            cards: Vec::new()
        }
    }

    pub fn load(&mut self) {
        self.cards.clear();
        for color in Color::ALL {
            for value in Value::ALL {
                self.cards.push(Card::new(value, color))
            }
        }
    }

    pub fn reload(&mut self) {
        self.load();
        self.shuffle();
    }

    pub fn shuffle(&mut self){
        self.cards.shuffle(&mut rand::thread_rng())
    }

    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }
}