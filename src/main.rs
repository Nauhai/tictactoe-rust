use tictactoe::*;
use rand::seq::SliceRandom;
use std::io::stdin;

fn main() {
    let interface = ConsoleInterface {};
    run(&interface);
}

pub struct ConsoleInterface {}
impl Interface for ConsoleInterface {
    fn choose_first_player_sign(&self) -> Sign {
        *[Sign::O, Sign::X].choose(&mut rand::thread_rng()).unwrap()
    }

    fn retrieve_input(&self, message: &str) -> String {
        println!("{}", message);
        let mut input = String::new();
        stdin().read_line(&mut input).expect("Failed to read input");
        input
    }

    fn on_play(&self, player: &Player, index: u8) {
        println!("{} plays on {}", player.sign, index);
    }

    fn show_board(&self, board: &Board) {
        println!("{}", board);
    }

    fn on_end(&self, game_state: GameState) {
        match game_state {
            GameState::Full => println!("Board is full, it's a draw."),
            GameState::Won(winner) => println!("{} Won the game!", winner),
            _ => panic!("Undefined behavior")
        }
    }
}
