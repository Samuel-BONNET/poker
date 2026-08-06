use crate::game::game::Game;
use shared::card::Card;
use shared::message::{GameSnapshot, PlayerSnapshot};

impl From<&Game> for GameSnapshot {
    fn from(game: &Game) -> Self {
        GameSnapshot {
            pot: game.pot + game.players.iter().map(|p| p.bet).sum::<i32>(),
            max_bet: game.max_bet,
            current_player: game.current_player,
            dealer_index: game.dealer_index,
            moment: format!("{:?}", game.moment),
            common_card: game.common_card.clone(),
            started: game.started,
            leader_seat: None,
            last_winner: game.last_winner.clone(),
            run_out: game.run_out_active(),
            players: game.players.iter().map(|p| PlayerSnapshot {
                id: p.id,
                name: p.name.clone(),
                bankroll: p.bankroll,
                bet: p.bet,
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