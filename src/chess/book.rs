use std::collections::HashMap;
use crate::chess::board;
use crate::chess::board::{Board, Side};
use crate::chess::moves::Move;
use crate::chess::moves_generation::MoveGenerator;

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
        match side {
            board::WHITE => Self::search_side(&self.white, turn, position),
            board::BLACK => Self::search_side(&self.black, turn, position),
            _ => panic!("wrong side: {}", side),
        }
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
            let legal_moves = board.generate_moves(false);

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
            let legal_moves = board.generate_moves(false);

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
    // Evans Gambit
    &[ "e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "f8c5", "b2b4" ],

    // Ruy Lopez
    &[ "e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "a7a6", "b5a4" ],

    // London
    &[ "d2d4", "g8f6", "c1f4" ], // Transposes to the main line below, order does not matter
    &[ "d2d4", "d7d5", "c1f4", "g8f6", "e2e3" ],
];

const OPENINGS_BLACK: &[&[&str]] = &[
    // Why is this even here?
    &[ "e2e4", "e7e5" ],

    // Caro-Kann
    &[ "e2e4", "c7c6", "d2d4", "d7d5", "e4e5", "c8f5" ], // Advance variation
    &[ "e2e4", "c7c6", "g1f3", "d7d5" ], // Knight bullshittery

    // Slav Defense
    &[ "d2d4", "d7d5", "c2c4", "c7c6" ],
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
