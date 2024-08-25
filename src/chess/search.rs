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
    use super::*;
    use crate::chess::moves_generation::MoveGenerator;
    use crate::chess::board::{BLACK, WHITE};

    pub trait SearchImpl {
        fn search_impl(&mut self, options: Options) -> SearchResult;

        fn eval(&mut self, m: Move) -> i64;
    }

    impl SearchImpl for Board {
        fn search_impl(&mut self, _: Options) -> SearchResult {
            let side = self.side_to_move();

            let mut moves = self.generate_moves();
            self.prune_checks(side, &mut moves);

            let modifier = if side == WHITE { 1i64 } else { -1i64 };

            let mut best = moves.0[0].clone();
            let mut best_eval = 0;

            for m in moves.0 {
                let eval = modifier * self.eval(m.clone());
                if eval > best_eval {
                    best = m.clone();
                    best_eval = eval;
                }
            }

            result!(best.clone(), modifier * best_eval, 1)
        }

        fn eval(&mut self, m: Move) -> i64 {
            let mut score = 0i64;
            self.make_move(m);

            for (side, modifier) in [(WHITE, 1i64), (BLACK, -1i64)] {
                let opponent = if side == WHITE { BLACK } else { WHITE };
                score += modifier * (self.pawns[side].count_ones() * 100) as i64;
                score += modifier * (self.knights[side].count_ones() * 300) as i64;
                score += modifier * (self.bishops[side].count_ones() * 320) as i64;
                score += modifier * (self.rooks[side].count_ones() * 500) as i64;
                score += modifier * (self.queens[side].count_ones() * 900) as i64;
                score += modifier * (if self.in_check(opponent) { 200} else { 0 });
            }

            self.unmake_move();

            score
        }
    }
}
