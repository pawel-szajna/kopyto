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
    pub depth: Option<usize>,
}

impl Options {
    pub fn new() -> Self {
        Self {
            white_time: u64::MAX,
            black_time: u64::MAX,
            white_increment: 0,
            black_increment: 0,
            depth: Some(4),
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
    use crate::chess::board::{BLACK, WHITE};
    use crate::chess::moves_generation::MoveGenerator;
    use rand::prelude::SliceRandom;

    const NULL_MOVE: Move = Move::new();

    pub trait SearchImpl {
        fn search_impl(&mut self, options: Options) -> SearchResult;

        fn order_moves(&mut self, moves: &mut Vec<Move>);
        fn eval(&mut self) -> i64;
        fn negamax(&mut self, depth: usize, alpha: i64, beta: i64) -> (Move, i64);
    }

    impl SearchImpl for Board {
        fn search_impl(&mut self, options: Options) -> SearchResult {
            let (m, eval) = self.negamax(options.depth.unwrap_or(usize::MAX), i64::MIN + 1, i64::MAX);
            result!(m, -eval, options.depth.unwrap_or(usize::MAX) as u64)
        }

        fn order_moves(&mut self, moves: &mut Vec<Move>) {
            let side = self.side_to_move();
            let opponent = if side == WHITE { BLACK } else { WHITE };
            let mut rng = rand::thread_rng();
            moves.shuffle(&mut rng);
            let attacks = self.get_attacks(side);
            moves.sort_by(|x, y| {
                match (1u64 << x.get_to()) & attacks == 0 && (1u64 << y.get_to()) & attacks != 0 {
                    true => std::cmp::Ordering::Less,
                    false => std::cmp::Ordering::Greater,
                }
            })
        }

        fn eval(&mut self) -> i64 {
            let mut score = 0i64;

            for (side, modifier) in [(WHITE, 1i64), (BLACK, -1i64)] {
                score += modifier * (self.pawns[side].count_ones() * 100) as i64;
                score += modifier * (self.knights[side].count_ones() * 300) as i64;
                score += modifier * (self.bishops[side].count_ones() * 320) as i64;
                score += modifier * (self.rooks[side].count_ones() * 500) as i64;
                score += modifier * (self.queens[side].count_ones() * 900) as i64;
            }

            score
        }

        fn negamax(&mut self, depth: usize, mut alpha: i64, beta: i64) -> (Move, i64) {
            let side = self.side_to_move();
            let multiplier = if side == WHITE { 1 } else { -1 };

            if depth == 0 {
                return (NULL_MOVE, self.eval() * multiplier);
            }

            let (mut moves, _) = self.generate_moves();
            self.prune_checks(self.side_to_move(), &mut moves);

            if moves.is_empty() {
                return (
                    NULL_MOVE,
                    if self.in_check(self.side_to_move()) {
                        (self.eval() - multiplier * 100000) * multiplier
                    } else {
                        0
                    },
                );
            }

            let mut best = NULL_MOVE;
            let mut best_eval = i64::MIN + 1;

            self.order_moves(&mut moves);

            for m in moves {
                self.make_move(m.clone());
                let (_, mut score) = self.negamax(depth - 1, -beta, -alpha);
                self.unmake_move();

                score = -score;

                if score > best_eval {
                    best = m;
                    best_eval = score;

                    if score > alpha {
                        alpha = score;
                    }
                }

                if score >= beta {
                    return (best, best_eval);
                }
            }

            (best, best_eval)
        }
    }
}
