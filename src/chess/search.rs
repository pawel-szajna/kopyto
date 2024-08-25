use crate::chess::board::Board;
use crate::chess::moves::Move;

pub struct SearchResult {
    pub m: Move,
    pub score: i64,
    pub depth: u64,
}

pub struct Options {
    pub white_time: u64,
    pub black_time: u64,
    pub white_increment: u64,
    pub black_increment: u64,
}

impl Options {
    pub fn new() -> Self {
        Self {
            white_time: u64::MAX,
            black_time: u64::MAX,
            white_increment: 0,
            black_increment: 0,
        }
    }
}

impl SearchResult {
    pub fn new(m: Move, score: i64, depth: u64) -> Self {
        Self { m, score, depth }
    }
}

macro_rules! result {
    ($m:expr,$s:expr,$d:expr) => {
        SearchResult::new($m, $s, $d)
    };
}

pub trait Search: pimpl::SearchImpl {
    fn search(&mut self, options: Options) -> SearchResult {
        self.search_impl(options)
    }
}

impl Search for Board {}

mod pimpl {
    use rand::prelude::SliceRandom;
    use super::*;
    use crate::chess::moves_generation::MoveGenerator;
    use crate::chess::board::{BLACK, WHITE};

    const NULL_MOVE: Move = Move::new();

    pub trait SearchImpl {
        fn search_impl(&mut self, options: Options) -> SearchResult;

        fn order_moves(&self, moves: &mut Vec<Move>);
        fn eval(&mut self) -> i64;
        fn negamax(&mut self, depth: usize) -> (Move, i64);
    }

    impl SearchImpl for Board {
        fn search_impl(&mut self, _: Options) -> SearchResult {
            let (m, eval) = self.negamax(3);
            result!(m, -eval, 3)
        }

        fn order_moves(&self, moves: &mut Vec<Move>) {
            let mut rng = rand::thread_rng();
            moves.shuffle(&mut rng);
        }

        fn eval(&mut self) -> i64 {
            let mut score = 0i64;

            for (side, modifier) in [(WHITE, 1i64), (BLACK, -1i64)] {
                let opponent = if side == WHITE { BLACK } else { WHITE };
                score += modifier * (self.pawns[side].count_ones() * 100) as i64;
                score += modifier * (self.knights[side].count_ones() * 300) as i64;
                score += modifier * (self.bishops[side].count_ones() * 320) as i64;
                score += modifier * (self.rooks[side].count_ones() * 500) as i64;
                score += modifier * (self.queens[side].count_ones() * 900) as i64;
                score += modifier * (if self.in_checkmate(opponent) { 100000 } else { 0 });
            }

            score
        }

        fn negamax(&mut self, depth: usize) -> (Move, i64) {
            if depth == 0 {
                return (NULL_MOVE, self.eval() * if self.side_to_move() == WHITE { 1 } else { -1 });
            }

            let mut moves = self.generate_moves();
            self.prune_checks(self.side_to_move(), &mut moves);
            let (mut moves, _) = moves;

            let mut best = NULL_MOVE;
            let mut best_eval = i64::MIN + 1;

            self.order_moves(&mut moves);

            for m in moves {
                self.make_move(m.clone());
                let (_, mut score) = self.negamax(depth - 1);
                score = -score;
                if score > best_eval {
                    best = m;
                    best_eval = score;
                }
                self.unmake_move();
            }

            (best, best_eval)
        }
    }
}
