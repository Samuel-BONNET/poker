use crate::Game;
use shared::card::Card;
use shared::message::{GameSnapshot, PlayerSnapshot};

impl From<&Game> for GameSnapshot {
    fn from(game: &Game) -> Self {
        GameSnapshot {
            pot: game.pot,
            max_bet: game.max_bet,
            current_player: game.current_player,
            dealer_index: game.dealer_index,
            moment: format!("{:?}", game.moment),
            common_card: game.common_card.clone(),
            started: game.started,
            players: game.players.iter().map(|p| PlayerSnapshot {
                id: p.id,
                name: p.name.clone(),
                bankroll: p.bankroll,
                bet: p.bet,
                hand: Vec::new(),
                folded: p.folded,
                all_in: p.all_in,
                small_blind: p.small_blind,
                big_blind: p.big_blind,
                talked: p.talked,
                active: p.active,
            }).collect()
        }
    }
}

impl Game {
    pub fn player_hand(&self, seat: usize) -> Vec<Card> {
        self.players.get(seat).map(|p| p.hand.cards.clone()).unwrap_or_default()
    }
}