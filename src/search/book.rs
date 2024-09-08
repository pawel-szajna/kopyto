use std::collections::HashMap;
use std::sync::LazyLock;
use crate::board::Board;
use crate::types::{Move, Side};
use crate::moves_generation;

type PositionMap = HashMap<u64, Vec<Move>>;
type SideMap = HashMap<u32, PositionMap>;

pub struct Book {
    white: SideMap,
    black: SideMap,
}

impl Book {
    pub fn new() -> Self {
        Self {
            white: SideMap::new(),
            black: SideMap::new(),
        }
    }

    pub fn search(&self, side: Side, turn: u32, position: u64) -> Option<Vec<Move>> {
        Self::search_side(side.choose(&self.white, &self.black), turn, position)
    }

    fn search_side(map: &SideMap, turn: u32, position: u64) -> Option<Vec<Move>> {
        match map.get(&turn) {
            None => None,
            Some(position_map) => match position_map.get(&position) {
                None => None,
                Some(moves) => Some(moves.clone()),
            }
        }
    }

    fn add_line_white(&mut self, board: &mut Board, moves: Vec<Move>) {
        assert_eq!(moves.len() % 2, 1);
        let mut moves_to_cancel = 0;
        for m in moves {
            let legal_moves = moves_generation::generate_all(board);

            if !legal_moves.contains(&m) {
                panic!("move from opening line is illegal");
            }

            if moves_to_cancel % 2 == 0 {
                let key = board.key();

                let clock = board.full_moves_count;
                if !self.white.contains_key(&clock) {
                    self.white.insert(clock, PositionMap::new());
                }

                let position = self.white.get_mut(&clock).unwrap();
                if !position.contains_key(&key) {
                    position.insert(key, vec![]);
                }

                let pos_moves = position.get_mut(&key).unwrap();
                if !pos_moves.contains(&m) {
                    pos_moves.push(m);
                }
            }

            board.make_move(m);
            moves_to_cancel += 1;
        }

        for _ in 0..moves_to_cancel {
            board.unmake_move();
        }
    }

    fn add_line_black(&mut self, board: &mut Board, moves: Vec<Move>) {
        assert_eq!(moves.len() % 2, 0);
        let mut moves_to_cancel = 0;
        for m in moves {
            let legal_moves = moves_generation::generate_all(board);

            if !legal_moves.contains(&m) {
                panic!("move from opening line is illegal");
            }

            if moves_to_cancel % 2 == 1 {
                let key = board.key();

                let clock = board.full_moves_count;
                if !self.black.contains_key(&clock) {
                    self.black.insert(clock, PositionMap::new());
                }

                let position = self.black.get_mut(&clock).unwrap();
                if !position.contains_key(&key) {
                    position.insert(key, vec![]);
                }

                let pos_moves = position.get_mut(&key).unwrap();
                if !pos_moves.contains(&m) {
                    pos_moves.push(m);
                }
            }

            board.make_move(m);
            moves_to_cancel += 1;
        }

        for _ in 0..moves_to_cancel {
            board.unmake_move();
        }
    }
}

pub trait BookGenerator {
    fn prepare_book(&mut self) -> Book;
}

const OPENINGS_WHITE: &[&[&str]] = &[

    // -- e4 -- //

    // Russian
    &[ "e2e4", "e7e5", "g1f3", "g8f6", "f3e5", "f6e4", "d1e2" ], // Damiano variation
    &[ "e2e4", "e7e5", "g1f3", "g8f6", "f3e5", "d7d6", "e5f3" ],
    &[ "e2e4", "e7e5", "g1f3", "g8f6", "f3e5", "b8c6", "e5c6" ], // Stafford gambit

    // Phildor
    &[ "e2e4", "e7e5", "g1f3", "d7d6", "d2d4", "e5d4", "f3d4" ], // Exchange
    &[ "e2e4", "e7e5", "g1f3", "d7d6", "d2d4", "c8g4", "d4e5" ],

    // Two knights defense
    &[ "e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "g8f6", "d2d3" ], // Modern bishop opening

    // Giuoco Piano
    &[ "e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "f8c5", "b2b4" ], // Evans Gambit

    // Ruy Lopez
    &[ "e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "a7a6", "b5a4" ], // Morphy defense
    &[ "e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "g8f6", "e1g1" ], // Berlin defense
    &[ "e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "d7d6", "d2d4" ], // Steinitz defense
    &[ "e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "d7d6", "e1g1" ],
    &[ "e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "f8c5", "e1g1" ], // Classical
    &[ "e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "f8c5", "c2c3" ],

    // Scotch
    &[ "e2e4", "e7e5", "g1f3", "b8c6", "d2d4", "e5d4", "f3d4" ], // Exchange
    &[ "e2e4", "e7e5", "g1f3", "b8c6", "d2d4", "d7d6", "d4e5" ], // Black d6
    &[ "e2e4", "e7e5", "g1f3", "b8c6", "d2d4", "d7d6", "f1b5" ],
    &[ "e2e4", "e7e5", "g1f3", "b8c6", "d2d4", "d7d6", "d4d5" ],
    &[ "e2e4", "e7e5", "g1f3", "b8c6", "d2d4", "g8f6", "d4d5" ], // Black Nf6
    &[ "e2e4", "e7e5", "g1f3", "b8c6", "d2d4", "g8f6", "d4e5" ],
    &[ "e2e4", "e7e5", "g1f3", "b8c6", "d2d4", "g8f6", "b1c3" ],

    // Scandinavian
    &[ "e2e4", "d7d5", "e4d5", "d8d5", "b1c3", "d5d6", "f1e2" ],
    &[ "e2e4", "d7d5", "e4d5", "d8d5", "b1c3", "d5d8", "d2d4" ],
    &[ "e2e4", "d7d5", "e4d5", "d8d5", "b1c3", "d5a5", "d2d4" ],

    // Sicilian
    &[ "e2e4", "c7c5", "g1f3", "b8c6", "d2d4", "c5d4", "f3d4" ],
    &[ "e2e4", "c7c5", "g1f3", "d7d6", "d2d4", "c5d4", "f3d4" ],
    &[ "e2e4", "c7c5", "g1f3", "e7e6", "d2d4", "c5d4", "f3d4" ],

    // French
    &[ "e2e4", "e7e6", "d2d4", "d7d5", "b1c3", "c7c5", "e4d5" ],
    &[ "e2e4", "e7e6", "d2d4", "d7d5", "b1c3", "d5e4", "c3e4" ],
    &[ "e2e4", "e7e6", "d2d4", "d7d5", "b1d2", "c7c5", "e4d5" ],
    &[ "e2e4", "e7e6", "d2d4", "d7d5", "b1d2", "c7c5", "g1f3" ],
    &[ "e2e4", "e7e6", "d2d4", "d7d5", "b1d2", "d5e4", "d2e4" ],
    &[ "e2e4", "e7e6", "d2d4", "d7d5", "e4e5", "b8c6", "g1f3" ],

    // Caro-Kann
    &[ "e2e4", "c7c6", "d2d4", "d7d5", "e4e5", "c8f5", "g1f3" ],
    &[ "e2e4", "c7c6", "d2d4", "d7d5", "e4e5", "c8f5", "h2h4" ],
    &[ "e2e4", "c7c6", "d2d4", "d7d5", "b1c3", "g8f6", "e4e5" ],
    &[ "e2e4", "c7c6", "d2d4", "d7d5", "b1c3", "d5e4", "c3e4" ], // Main line
    &[ "e2e4", "c7c6", "d2d4", "d7d5", "e4d5", "c6d5", "c2c4" ], // Panov attack
    &[ "e2e4", "c7c6", "d2d4", "d7d5", "e4d5", "c6d5", "f1d3" ],

    // Pirc
    &[ "e2e4", "d7d6", "d2d4", "g8f6", "b1c3" ],

    // -- d4 -- //

    // London
    &[ "d2d4", "g8f6", "c1f4" ], // Transposes to the main line below, order does not matter
    &[ "d2d4", "d7d5", "c1f4", "b8c6", "e2e3" ],
    &[ "d2d4", "d7d5", "c1f4", "g8f6", "e2e3", "b8c6", "g1f3" ],
    &[ "d2d4", "d7d5", "c1f4", "g8f6", "e2e3", "e7e6", "g1f3" ],
    &[ "d2d4", "d7d5", "c1f4", "g8f6", "e2e3", "c7c5", "c2c3" ],


    // Zukertort
    &[ "d2d4", "d7d5", "g1f3", "b8c6", "c1f4", "g8f6", "e2e3" ],
    &[ "d2d4", "d7d5", "g1f3", "b8c6", "c1f4", "c8f5", "e2e3" ],
    &[ "d2d4", "d7d5", "g1f3", "b8c6", "c2c4", "e7e6", "b1c3" ],
    &[ "d2d4", "d7d5", "g1f3", "b8c6", "c2c4", "d5c4", "e2e3" ],
    &[ "d2d4", "d7d5", "g1f3", "b8c6", "c2c4", "d5c4", "b1c3" ],
    &[ "d2d4", "d7d5", "g1f3", "b8c6", "c2c4", "d5c4", "d4d5" ],

    // Queen's gambit
    &[ "d2d4", "d7d5", "c2c4" ],
    &[ "d2d4", "d7d5", "c2c4", "d5c4", "g1f3" ], // Accepted, normal
    &[ "d2d4", "d7d5", "c2c4", "d5c4", "e2e3" ], // Accepted, old
    &[ "d2d4", "d7d5", "c2c4", "d5c4", "e2e4" ], // Accepted, Saduleto
    &[ "d2d4", "d7d5", "c2c4", "c7c6", "g1f3" ], // Slav Defense
    &[ "d2d4", "d7d5", "c2c4", "e7e6", "b1c3", "d5c4", "e2e4" ], // Declined, dxc4 capture
    &[ "d2d4", "d7d5", "c2c4", "e7e6", "b1c3", "d5c4", "e2e3" ],
    &[ "d2d4", "d7d5", "c2c4", "e7e6", "b1c3", "g8f6", "c4d5" ], // Declined, normal line
    &[ "d2d4", "d7d5", "c2c4", "e7e6", "b1c3", "g8f6", "g1f3" ],
    &[ "d2d4", "d7d5", "c2c4", "e7e6", "b1c3", "g8f6", "c1g5" ],

    // Responses to 1. ...Nc6
    &[ "d2d4", "b8c6", "g1f3", "d7d5", "c1f4", "g8f6", "e2e3" ],
    &[ "d2d4", "b8c6", "g1f3", "d7d5", "c1f4", "c8f5", "e2e3" ],
    &[ "d2d4", "b8c6", "g1f3", "d7d5", "c1f4", "e7e6", "e2e3" ],
    &[ "d2d4", "b8c6", "g1f3", "d7d5", "c2c4", "d5c4", "e2e3" ],
    &[ "d2d4", "b8c6", "g1f3", "d7d5", "c2c4", "d5c4", "b1c3" ],
    &[ "d2d4", "b8c6", "g1f3", "d7d5", "c2c4", "e7e6", "b1c3" ],
    &[ "d2d4", "b8c6", "g1f3", "g8f6", "c2c4" ],
    &[ "d2d4", "b8c6", "g1f3", "g8f6", "d4d5", "c6b4", "c2c4" ],

];

const OPENINGS_BLACK: &[&[&str]] = &[

    // -- Against e4 -- //

    // Italian
    &[ "e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "f8c5", "d2d3", "g8f6" ], // Giuoco Piano
    &[ "e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "f8c5", "b2b4", "c5b4" ], // Evans Gambit
    &[ "e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "g8f6", "f3g5", "d7d5" ], // Fried liver

    // Ruy Lopez
    &[ "e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "a7a6", "b5c6", "d7c6" ], // Exchange
    &[ "e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "a7a6", "b5a4", "g8f6" ],

    // Four knights
    &[ "e2e4", "e7e5", "g1f3", "b8c6", "b1c3", "g8f6", "f1c4", "f6e4" ], // Italian-like
    &[ "e2e4", "e7e5", "g1f3", "b8c6", "b1c3", "g8f6", "f1c4", "f8c5" ],
    &[ "e2e4", "e7e5", "g1f3", "b8c6", "b1c3", "g8f6", "d2d4", "e5d4" ], // Scotch-like

    // Scotch
    &[ "e2e4", "e7e5", "g1f3", "b8c6", "d2d4", "e5d4", "f3d4", "f8c5" ],
    &[ "e2e4", "e7e5", "g1f3", "b8c6", "d2d4", "e5d4", "f3d4", "g8f6" ],
    &[ "e2e4", "e7e5", "g1f3", "b8c6", "d2d4", "e5d4", "f1c4", "g8f6" ],

    // Center game
    &[ "e2e4", "e7e5", "d2d4", "e5d4", "d1d4", "b8c6", "d4d1", "g8f6" ],
    &[ "e2e4", "e7e5", "d2d4", "e5d4", "d1d4", "b8c6", "d4d1", "f8c5" ],
    &[ "e2e4", "e7e5", "d2d4", "e5d4", "d1d4", "b8c6", "d4e3", "g8f6" ], // Paulsen attack
    &[ "e2e4", "e7e5", "d2d4", "e5d4", "c2c3", "d7d5", "e4d5", "d8d5" ], // Danish gambit declined
    &[ "e2e4", "e7e5", "d2d4", "e5d4", "c2c3", "d4c3", "b1c3", "b8c6" ], // Danish gambit accepted
    &[ "e2e4", "e7e5", "d2d4", "e5d4", "c2c3", "d4c3", "b1c3", "f8b4" ],
    &[ "e2e4", "e7e5", "d2d4", "e5d4", "c2c3", "d4c3", "f1c4", "c3b2" ],

    // Caro-Kann
    &[ "e2e4", "c7c6", "d2d4", "d7d5", "e4e5", "c8f5", "g1f3", "e7e6" ], // Advance variation
    &[ "e2e4", "c7c6", "d2d4", "d7d5", "e4e5", "c8f5", "f1d3", "f5d3" ],
    &[ "e2e4", "c7c6", "d2d4", "d7d5", "e4e5", "c8f5", "b1c3", "e7e6" ],
    &[ "e2e4", "c7c6", "d2d4", "d7d5", "e4e5", "c8f5", "g2g4", "f5e4" ],
    &[ "e2e4", "c7c6", "d2d4", "d7d5", "e4d5", "c6d5", "b1c3", "b8c6" ], // Exchange variation
    &[ "e2e4", "c7c6", "d2d4", "d7d5", "e4d5", "c6d5", "b1c3", "g8f6" ],
    &[ "e2e4", "c7c6", "d2d4", "d7d5", "e4d5", "c6d5", "g1f3", "b8c6" ],
    &[ "e2e4", "c7c6", "d2d4", "d7d5", "e4d5", "c6d5", "g1f3", "g8f6" ],
    &[ "e2e4", "c7c6", "d2d4", "d7d5", "e4d5", "c6d5", "c2c4", "g8f6" ], // Panov attack
    &[ "e2e4", "c7c6", "f1c4", "d7d5", "c4b3", "d5e4" ], // Hillbilly attack
    &[ "e2e4", "c7c6", "g1f3", "d7d5", "b1c3", "c8g4" ], // Two knights attack
    &[ "e2e4", "c7c6", "g1f3", "d7d5", "e4d5", "c6d5" ],
    &[ "e2e4", "c7c6", "b1c3", "d7d5" ], // Queen-side knight first

    // -- Against d4 -- //

    // Slav Defense
    &[ "d2d4", "d7d5", "c2c4", "c7c6", "b1c3", "g8f6", "g1f3", "e7e6" ],
    &[ "d2d4", "d7d5", "c2c4", "c7c6", "g1f3", "g8f6" ],

    // Queen's Gambit Declined

    &[ "d2d4", "d7d5", "c2c4", "e7e6", "b1c3", "g8f6" ],
    &[ "d2d4", "d7d5", "c2c4", "e7e6", "g1f3", "g8f6" ],

    // Anti-London
    &[ "d2d4", "d7d5", "c1f4", "g8f6", "e2e3", "c7c5" ], // Main (?) line
    &[ "d2d4", "d7d5", "c1f4", "g8f6", "g1f3", "c7c5" ], // Knight first, supposed to be bad
    &[ "d2d4", "d7d5", "c1f4", "g8f6", "b1c3", "e7e6" ], // Jobava-London
    &[ "d2d4", "d7d5", "c1f4", "g8f6", "b1c3", "a7a6" ], // Alternative Jobava line

    // -- Against e3 -- //

    &[ "e2e3", "d7d5", "d2d4", "g8f6", "g1f3", "e7e6" ], // Colle system with strange move order?

    // -- Against Nf3 -- //

    &[ "g1f3", "d7d5", "d2d4", "g8f6" ],
    &[ "g1f3", "g8f6", "d2d4", "g7g6", "c2c4", "f8g7" ],
];

impl BookGenerator for Board {
    fn prepare_book(&mut self) -> Book {
        let mut book = Book::new();

        for line in OPENINGS_WHITE {
            book.add_line_white(self, line.iter().map(|x| Move::from_uci(x)).collect::<Vec<Move>>());
        }

        for line in OPENINGS_BLACK {
            book.add_line_black(self, line.iter().map(|x| Move::from_uci(x)).collect::<Vec<Move>>());
        }

        book
    }
}

pub static BOOK: LazyLock<Book> = LazyLock::new(|| Board::from_starting_position().prepare_book());
