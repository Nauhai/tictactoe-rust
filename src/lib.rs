use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt::{self, Display, Formatter};


#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Sign {
    O, X
}

impl Display for Sign {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Sign::O => "O",
            Sign::X => "X",
        })
    }
}


#[derive(Debug, PartialEq)]
pub enum TileState {
    Marked(Sign),
    Empty,
}

impl Display for TileState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            TileState::Marked(sign) => write!(f, "{}", sign),
            TileState::Empty => write!(f, " ")
        }
    }
}


#[derive(Debug, PartialEq)]
pub enum GameState {
    NotOver,
    Full,
    Won(Sign)
}


#[derive(Debug, PartialEq)]
pub struct Board {
    tiles: HashMap<u8, TileState>,
}

impl Board {
    pub fn new() -> Board {
        let mut tiles = HashMap::new();
        (1..=9).for_each(|i| {tiles.insert(i, TileState::Empty);});
        Board { tiles }
    }

    fn from_str(sequence: &str) -> Board {
        if sequence.len() != 9 {
            panic!("Uncomputable sequence")
        }

        let tiles = (1..=9).into_iter().zip(
            sequence.chars()
                .map(|c| match c {
                    'O' => TileState::Marked(Sign::O),
                    'X' => TileState::Marked(Sign::X),
                    ' ' => TileState::Empty,
                    other => panic!("Unknown tile identifier: '{}'", other)
                })
        ).collect::<HashMap<u8, TileState>>();

        Board { tiles }
    }

    pub fn set_tile(&mut self, index: u8, sign: Sign) -> Result<u8, &str> {
        match self.tiles.entry(index) {
            Entry::Occupied(mut entry) => match entry.get() {
                TileState::Empty => {
                    entry.insert(TileState::Marked(sign));
                    Ok(index)
                },
                TileState::Marked(_) => Err("This tile is already marked. Please try another tile")
            },
            _ => panic!("Undefined behavior")
        }
    }

    pub fn is_full(&self) -> bool {
        self.tiles.values().all(|v| matches!(v, TileState::Marked(_)))
    }

    pub fn get_winner(&self) -> Option<Sign> {
        let layouts = [
            [1, 2, 3],
            [4, 5, 6],
            [7, 8, 9],
            [1, 4, 7],
            [2, 5, 8],
            [3, 6, 9],
            [1, 5, 9],
            [3, 5, 7]
        ];

        for layout in layouts {
            let mut signs = layout.into_iter().map(|i| self.tiles.get(&i).unwrap());
        
            match (signs.next().unwrap(), signs.next().unwrap(), signs.next().unwrap()) {
                (TileState::Marked(s1), TileState::Marked(s2), TileState::Marked(s3)) => {
                    if s1 == s2 && s2 == s3 {
                        return Some(*s1)
                    }
                },
                _ => continue
            }
        } 

        None
    }

    pub fn get_game_state(&self) -> GameState {
        if let Some(winner) = self.get_winner() {
            GameState::Won(winner)
        } else if self.is_full() {
            GameState::Full
        } else {
            GameState::NotOver
        }
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f,
            " {} | {} | {} \n-----------\n {} | {} | {} \n-----------\n {} | {} | {}",
            self.tiles[&1],
            self.tiles[&2],
            self.tiles[&3],
            self.tiles[&4],
            self.tiles[&5],
            self.tiles[&6],
            self.tiles[&7],
            self.tiles[&8],
            self.tiles[&9],
        )
    }
}


#[derive(Debug)]
pub struct Player {
    pub sign: Sign
}


pub trait Interface {
    fn choose_first_player<'a>(&self, players: &'a [Player]) -> &'a Player;
    fn retrieve_input(&self, message: &str) -> String;
    fn on_play(&self, player: &Player, index: u8);
    fn show_board(&self, board: &Board);
    fn on_end(&self, game_state: GameState);
}



pub fn run<T: Interface>(interface: &T) {
    let players = [
        Player { sign: Sign::O },
        Player { sign: Sign::X }
    ];
    let mut current_player = interface.choose_first_player(&players);

    let mut board = Board::new();
    
    while board.get_game_state() == GameState::NotOver {
        interface.show_board(&board);
        player_moves(current_player, &mut board, interface);

        current_player = match current_player.sign {
            Sign::O => &players[1],
            Sign::X => &players[0],
        };
    }

    interface.on_end(board.get_game_state());
    interface.show_board(&board);
}

pub fn player_moves<T: Interface>(player: &Player, board: &mut Board, interface: &T) {
    let mut input = interface.retrieve_input(&format!("{}, please enter a tile number (1-9):", player.sign));
    loop {
        match validate_input(&input).and_then(|i| board.set_tile(i, player.sign)) {
            Ok(index) => break interface.on_play(player, index),
            Err(e) => input = interface.retrieve_input(e)
        }
    }
}

pub fn validate_input(input: &String) -> Result<u8, &str> {
    match input.trim().parse::<u8>() {
        Ok(n) if (1..=9).contains(&n) => Ok(n),
        _ => Err("Please enter a number between 1 and 9")
    }
}



#[cfg(test)]
mod tests {
    use crate::{*, Sign::*, TileState::*, GameState::*};
    use std::collections::{HashMap, VecDeque};
    use std::cell::RefCell;


    struct MockInterface<'a> {
        inputs: RefCell<VecDeque<&'a str>>,
        actions: RefCell<Vec<String>>,
    }

    impl<'a> MockInterface<'a> {
        fn new(inputs: Vec<&'a str>) -> MockInterface<'a> {
            MockInterface {
                inputs: RefCell::new(VecDeque::from(inputs)),
                actions: RefCell::new(vec![]),
            }
        }
    }

    impl<'a> Interface for MockInterface<'a> {
        fn choose_first_player<'b>(&self, players: &'b [Player]) -> &'b Player {
            return &players[0]
        }

        fn retrieve_input(&self, message: &str) -> String {
            match self.inputs.borrow_mut().pop_front() {
                Some(input) => input.to_string(),
                None => panic!("No more input registered")
            }
        }

        fn on_play(&self, player: &Player, index: u8) {
            self.actions.borrow_mut().push(format!("{} plays on {}", player.sign, index))
        }

        fn show_board(&self, board: &Board) {}

        fn on_end(&self, game_state: GameState) {
            self.actions.borrow_mut().push(match game_state {
                GameState::Full => "Draw, board is full".to_string(),
                GameState::Won(winner) => format!("Game won by {}", winner),
                _ => "Error".to_string()
            });
        }
    }


    #[test]
    fn from_str_works() {
        let tiles = "OX X OO X";
        let board = Board::from_str(tiles);

        let should_be = HashMap::from([
            (1, Marked(O)),
            (2, Marked(X)),
            (3, Empty),
            (4, Marked(X)),
            (5, Empty),
            (6, Marked(O)),
            (7, Marked(O)),
            (8, Empty),
            (9, Marked(X)),
        ]);

        assert_eq!(board.tiles, should_be);
    }

    #[test]
    fn horizontal_win() {
        let board1 = Board::from_str("OOO      ");
        let board2 = Board::from_str("   OOO   ");
        let board3 = Board::from_str("      OOO");

        assert_eq!(board1.get_winner(), Some(O));
        assert_eq!(board2.get_winner(), Some(O));
        assert_eq!(board3.get_winner(), Some(O));

        assert_eq!(board1.get_game_state(), Won(O));
        assert_eq!(board2.get_game_state(), Won(O));
        assert_eq!(board3.get_game_state(), Won(O));
    }

    #[test]
    fn vertical_win() {
        let board1 = Board::from_str("O  O  O  ");
        let board2 = Board::from_str(" O  O  O ");
        let board3 = Board::from_str("  O  O  O");

        assert_eq!(board1.get_winner(), Some(O));
        assert_eq!(board2.get_winner(), Some(O));
        assert_eq!(board3.get_winner(), Some(O));

        assert_eq!(board1.get_game_state(), Won(O));
        assert_eq!(board2.get_game_state(), Won(O));
        assert_eq!(board3.get_game_state(), Won(O));
    }

    #[test]
    fn diagonal_win() {
        let board1 = Board::from_str("O   O   O");
        let board2 = Board::from_str("  O O O  ");

        assert_eq!(board1.get_winner(), Some(O));
        assert_eq!(board2.get_winner(), Some(O));

        assert_eq!(board1.get_game_state(), Won(O));
        assert_eq!(board2.get_game_state(), Won(O));
    }

    #[test]
    fn board_full() {
        let board = Board::from_str("OXOXXOXOX");

        assert!(board.is_full());
        assert_eq!(board.get_game_state(), Full);
    }

    #[test]
    fn board_not_full() {
        let board = Board::from_str("OXOX OXOX");

        assert!(!board.is_full());
        assert_eq!(board.get_game_state(), NotOver);
    }

    #[test]
    fn it_sets_tile() {
        let mut board = Board::new();

        assert!(board.set_tile(1, O).is_ok());
        assert_eq!(*board.tiles.get(&1).unwrap(), Marked(O));
    }

    #[test]
    fn it_doesnt_set_tile() {
        let mut board = Board::from_str("O        ");

        assert!(board.set_tile(1, X).is_err());
        assert_eq!(*board.tiles.get(&1).unwrap(), Marked(O));
    }
    
    #[test]
    fn input_correctly_validated() {
        for i in 1..=9 {
            assert!(validate_input(&i.to_string()).is_ok())
        }
    }

    #[test]
    fn input_correctly_invalidated() {
        assert!(validate_input(&String::from("0")).is_err());
        assert!(validate_input(&String::from("10")).is_err());
        assert!(validate_input(&String::from("yo")).is_err());
    }
    
    #[test]
    fn it_draws() {
        let mock = MockInterface::new(vec!["5", "2", "6", "4", "1", "9", "7", "3", "8"]);
        run(&mock);

        let actions = vec![
            "O plays on 5".to_string(),
            "X plays on 2".to_string(),
            "O plays on 6".to_string(),
            "X plays on 4".to_string(),
            "O plays on 1".to_string(),
            "X plays on 9".to_string(),
            "O plays on 7".to_string(),
            "X plays on 3".to_string(),
            "O plays on 8".to_string(),
            "Draw, board is full".to_string()
        ];

        assert_eq!(mock.actions.borrow().to_owned(), actions);
    }

    #[test]
    fn it_wins() {
        let mock1 = MockInterface::new(vec!["1", "2", "4", "5", "7", "8"]);
        run(&mock1);

        let mock2 = MockInterface::new(vec!["2", "1", "3", "5", "4", "9"]);
        run(&mock2);

        assert_eq!(mock1.actions.borrow().last().unwrap(), "Game won by O");
        assert_eq!(mock2.actions.borrow().last().unwrap(), "Game won by X");
    }

}
