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

mod weights {
    type Weights = [i64; 64];
    type WeightsPerSide = [Weights; 2];

    const PAWN_BASE: Weights = [
        0, 0, 0, 0, 0, 0, 0, 0,
        25, 35, 30, 25, 25, 30, 35, 25,
        20, 30, 22, 24, 24, 22, 30, 20,
        10, 10, 20, 35, 35, 20, 10, 10,
        5, 5, 15, 36, 36, 15, 5, 5,
        4, 5, 10, 14, 14, 10, 5, 4,
        0, 0, 0, -4, -4, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
    ];

    const KNIGHT_BASE: Weights = [
        -100, -84, -75, -70, -70, -75, -84, -100,
        -82, -45, -5, -5, -5, -5, -48, -82,
        -76, -24, 16, 32, 32, 16, -24, -76,
        -72, 0, 32, 48, 48, 32, 0, -72,
        -72, 0, 32, 48, 48, 32, 0, -72,
        -74, -2, 26, 24, 24, 26, -2, -74,
        -80, -16, 0, 4, 4, 0, -16, -80,
        -100, -12, -24, -48, -48, -24, -12, -100
    ];

    const KING_BASE: Weights = [
        -88, -88, -88, -88, -88, -88, -88, -88,
        -77, -77, -77, -77, -77, -77, -77, -77,
        -62, -62, -62, -62, -62, -62, -62, -62,
        -44, -44, -44, -44, -44, -44, -44, -44,
        -17, -19, -23, -36, -33, -23, -19, -17,
        -8, -8, -19, -29, -27, -17, -8, -8,
        0, 1, -2, -60, -50, -8, 5, 4,
        -8, 48, 20, -55, 10, -20, 44, 17
    ];

    const BISHOP_BASE: Weights = [
        -29, 4, -82, -37, -25, -42, 7, -8,
        -26, 16, -18, -13, 30, 59, 18, -47,
        -16, 37, 43, 40, 35, 50, 37, -2,
        -4, 5, 19, 50, 37, 37, 7, -2,
        -6, 13, 13, 26, 34, 12, 10, 4,
        0, 15, 15, 15, 14, 27, 18, 10,
        4, 15, 16, 0, 7, 21, 33, 1,
        -33, -3, -14, -21, -13, -12, -39, -21,
    ];

    const ROOK_BASE: Weights = [
        32, 42, 32, 51, 63, 9, 31, 43,
        27, 32, 58, 62, 80, 67, 26, 44,
        -5, 19, 26, 36, 17, 45, 61, 16,
        -24, -11, 7, 26, 24, 35, -8, -20,
        -36, -26, -12, -1, 9, -7, 6, -23,
        -45, -25, -16, -17, 3, 0, -5, -33,
        -44, -16, -20, -9, -1, 11, -6, -71,
        -19, -13, 1, 17, 16, 7, -37, -26,
    ];

    const QUEEN_BASE: Weights = [
        -28, 0, 29, 12, 59, 44, 43, 45,
        -24, -39, -5, 1, -16, 57, 28, 54,
        -13, -17, 7, 8, 29, 56, 47, 57,
        -27, -27, -16, -16, -1, 17, -2, 1,
        -9, -26, -9, -10, -2, -4, 3, -3,
        -14, 2, -11, -2, -5, 2, 14, 5,
        -35, -8, 11, 2, 8, 15, -3, 1,
        -1, -18, -9, 10, -15, -25, -31, -50,
    ];

    macro_rules! rev_weights {
    ($x:ident) => { [
        $x[56 + 0], $x[56 + 1], $x[56 + 2], $x[56 + 3], $x[56 + 4], $x[56 + 5], $x[56 + 6], $x[56 + 7],
        $x[48 + 0], $x[48 + 1], $x[48 + 2], $x[48 + 3], $x[48 + 4], $x[48 + 5], $x[48 + 6], $x[48 + 7],
        $x[40 + 0], $x[40 + 1], $x[40 + 2], $x[40 + 3], $x[40 + 4], $x[40 + 5], $x[40 + 6], $x[40 + 7],
        $x[32 + 0], $x[32 + 1], $x[32 + 2], $x[32 + 3], $x[32 + 4], $x[32 + 5], $x[32 + 6], $x[32 + 7],
        $x[24 + 0], $x[24 + 1], $x[24 + 2], $x[24 + 3], $x[24 + 4], $x[24 + 5], $x[24 + 6], $x[24 + 7],
        $x[16 + 0], $x[16 + 1], $x[16 + 2], $x[16 + 3], $x[16 + 4], $x[16 + 5], $x[16 + 6], $x[16 + 7],
        $x[8 + 0], $x[8 + 1], $x[8 + 2], $x[8 + 3], $x[8 + 4], $x[8 + 5], $x[8 + 6], $x[8 + 7],
        $x[0 + 0], $x[0 + 1], $x[0 + 2], $x[0 + 3], $x[0 + 4], $x[0 + 5], $x[0 + 6], $x[0 + 7],
    ]};
}

    macro_rules! double_weights {
    ($x:ident) => { [ rev_weights!($x), $x ] }
}

    pub const PAWN: WeightsPerSide = double_weights!(PAWN_BASE);
    pub const KNIGHT: WeightsPerSide = double_weights!(KNIGHT_BASE);
    pub const KING: WeightsPerSide = double_weights!(KING_BASE);
    pub const BISHOP: WeightsPerSide = double_weights!(BISHOP_BASE);
    pub const ROOK: WeightsPerSide = double_weights!(ROOK_BASE);
    pub const QUEEN: WeightsPerSide = double_weights!(QUEEN_BASE);
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
    use crate::chess::transpositions::Score::{Exact, LowerBound, UpperBound};

    const NULL_MOVE: Move = Move::new();

    struct SearchContext {
        pub position_history: Vec<u64>,
    }

    impl SearchContext {
        pub fn new() -> Self {
            Self {
                position_history: Vec::with_capacity(50),
            }
        }

        pub fn triple_repetition(&self) -> bool {
            match self.position_history.last() {
                None => false,
                Some(position) => self.position_history.iter().filter(|p| p == &position).count() == 3,
            }
        }
    }

    pub trait SearchImpl {
        fn search_impl(&mut self, options: Options) -> SearchResult;

        fn order_moves(&mut self, moves: &mut Vec<Move>);
        fn eval(&self) -> i64;
        fn eval_piece(&self, mask: u64, value: i64, weights: &[i64; 64]) -> i64;
        fn negamax(&mut self, context: &mut SearchContext, depth: usize, alpha: i64, beta: i64) -> (Move, i64);
    }

    impl SearchImpl for Board {
        fn search_impl(&mut self, options: Options) -> SearchResult {
            let mut context = SearchContext::new();
            let (m, eval) = self.negamax(&mut context, options.depth.unwrap_or(usize::MAX), i64::MIN + 1, i64::MAX);
            result!(m, -eval, options.depth.unwrap_or(usize::MAX) as u64)
        }

        fn order_moves(&mut self, moves: &mut Vec<Move>) {
            let side = self.side_to_move();
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

        fn eval(&self) -> i64 {
            let mut score = 0i64;

            for (side, modifier) in [(WHITE, 1i64), (BLACK, -1i64)] {
                score += modifier * self.eval_piece(self.kings[side], 0, &weights::KING[side]);
                score += modifier * self.eval_piece(self.pawns[side], 100, &weights::PAWN[side]);
                score += modifier * self.eval_piece(self.knights[side], 300, &weights::KNIGHT[side]);
                score += modifier * self.eval_piece(self.bishops[side], 320, &weights::BISHOP[side]);
                score += modifier * self.eval_piece(self.rooks[side], 500, &weights::ROOK[side]);
                score += modifier * self.eval_piece(self.queens[side], 900, &weights::QUEEN[side]);
            }

            score
        }

        fn eval_piece(&self, mut mask: u64, value: i64, weights: &[i64; 64]) -> i64 {
            let mut score = 0;
            while mask != 0 {
                let pos = mask.trailing_zeros() as usize;
                score += value + weights[pos];
                mask ^= 1u64 << pos;
            }
            score
        }

        fn negamax(&mut self, context: &mut SearchContext, depth: usize, mut alpha: i64, beta: i64) -> (Move, i64) {
            let side = self.side_to_move();
            let multiplier = if side == WHITE { 1 } else { -1 };

            let key = self.key();
            match self.transpositions.get(key, depth, alpha, beta) {
                Some((score, m)) => return (m, score),
                None => (),
            }

            if context.triple_repetition() {
                return (NULL_MOVE, 0);
            }

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

            let mut found_exact = false;
            let mut best_move = 0;

            for m in moves {
                self.make_move(m.clone());
                let key = self.key();
                context.position_history.push(key);
                let (_, mut score) = self.negamax(context, depth - 1, -beta, -alpha);
                context.position_history.pop();
                self.unmake_move();

                score = -score;

                if score > best_eval {
                    best = m;
                    best_eval = score;

                    if score > alpha {
                        found_exact = true;
                        best_move = key;
                        alpha = score;
                    }
                }

                if score >= beta {
                    self.transpositions.set(key, depth, LowerBound(beta), m);
                    return (best, best_eval);
                }
            }

            self.transpositions.set(key, depth, if found_exact { Exact(alpha) } else { UpperBound(alpha) }, best);

            (best, best_eval)
        }
    }
}
