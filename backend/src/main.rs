mod card;
mod game;
mod player;
mod action;
mod moment;

mod hand;

use game::Game;
use player::Player;

fn main() {
    println!("Play Poker !");

    const BANKROLL: i32 = 200;

    // setup player
    let player1 = Player::new(1, "Player1".to_string(), BANKROLL);
    let player2 = Player::new(2, "Player2".to_string(), BANKROLL);
    let player3 = Player::new(3, "Player3".to_string(), BANKROLL);
    let player4 = Player::new(4, "Player4".to_string(), BANKROLL);


    // init game + link players
    let mut game = Game::new();
    game.add_player(player1);
    game.add_player(player2);
    game.add_player(player3);
    game.add_player(player4);

    // start game
    game.run()
}
