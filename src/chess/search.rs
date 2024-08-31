use crate::chess::board::Board;
use crate::chess::moves::Move;

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

pub struct Options {
    pub white_time: u64,
    pub black_time: u64,
    #[allow(dead_code)]
    pub white_increment: u64,
    #[allow(dead_code)]
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
            depth: None,
        }
    }
}

pub trait Search: pimpl::SearchImpl {
    fn search(&mut self, options: Options) -> Move {
        self.search_impl(options)
    }
}

impl Search for Board {}

mod pimpl {
    use std::cmp::max;
    use std::time::SystemTime;
    use super::*;
    use crate::chess::board::{BLACK, WHITE};
    use crate::chess::moves_generation::MoveGenerator;
    use rand::prelude::SliceRandom;
    use crate::chess::moves::Piece;
    use crate::chess::transpositions::Score::{Exact, LowerBound, UpperBound};
    use crate::chess::util;

    const NULL_MOVE: Move = Move::new();
    const MIN_MOVE_TIME: u128 = 200;

    pub struct SearchContext {
        depth: usize,
        seldepth: usize,
        best_move: Move,
        nodes: u64,
        start_time: SystemTime,
        target_time: u128,
        time_hit: bool,
    }

    impl SearchContext {
        pub fn new(depth: usize, start_time: SystemTime, target_time: u128) -> Self {
            Self {
                depth,
                seldepth: 0,
                best_move: NULL_MOVE,
                nodes: 0,
                start_time,
                target_time,
                time_hit: false,
            }
        }
    }

    pub trait SearchImpl {
        fn search_impl(&mut self, options: Options) -> Move;

        fn order_moves(&mut self, moves: &mut Vec<Move>);
        fn eval(&self) -> i64;
        fn eval_piece(&self, mask: u64, value: i64, weights: &[i64; 64]) -> i64;
        fn negamax(&mut self, context: &mut SearchContext, depth: usize, alpha: i64, beta: i64) -> i64;
        fn qsearch(&mut self, context: &mut SearchContext, depth_in: usize, alpha: i64, beta: i64) -> i64;
    }

    impl SearchImpl for Board {
        fn search_impl(&mut self, options: Options) -> Move {
            let start = SystemTime::now();
            self.transpositions.clear();

            let side = self.side_to_move();
            let our_time = if side == WHITE { options.white_time } else { options.black_time };
            let opponent_time = if side == WHITE { options.black_time } else { options.white_time };
            let time_advantage = our_time as i64 - opponent_time as i64;
            let time_advantage_modifier = if time_advantage > 0 { time_advantage / 4 } else { time_advantage / 8 };

            let target_time = our_time / 40 + max(0, time_advantage_modifier) as u64;

            println!("info string our_time {} opponent_time {} time_advantage {} advantage_modifier {} target_time {}", our_time ,opponent_time, time_advantage, time_advantage_modifier, target_time);

            let depth = options.depth.unwrap_or(usize::MAX);
            let mut context = SearchContext::new(depth, start, max(target_time as u128, MIN_MOVE_TIME));
            let mut eval = self.last_eval;

            for current_depth in 1..=depth {
                context.depth = current_depth;
                let last_eval = eval;
                let window_size = 40;
                eval = self.negamax(&mut context, current_depth, last_eval - window_size, last_eval + window_size);
                if context.time_hit {
                    break;
                }
                if (last_eval - eval).abs() >= window_size {
                    eval = self.negamax(&mut context, current_depth, i64::MIN + 1, i64::MAX);
                    if context.time_hit {
                        break;
                    }
                }
                let abs_eval = if side == WHITE { eval } else { -eval };

                let time_taken = context.start_time.elapsed().unwrap();
                println!(
                    "info depth {} seldepth {} score {} nodes {} nps {} time {} hashfull {}",
                    current_depth,
                    context.seldepth,
                    util::eval_to_str(abs_eval),
                    context.nodes,
                    1000000000 * context.nodes as u128 / max(1, time_taken.as_nanos()),
                    time_taken.as_millis(),
                    self.transpositions.usage());

                if time_taken.as_millis() >= context.target_time {
                    break;
                }
            }

            if context.best_move == NULL_MOVE {
                println!("info string null move selected as best, bug?");
            }

            self.last_eval = -eval;
            context.best_move
        }

        fn order_moves(&mut self, moves: &mut Vec<Move>) {
            let mut rng = rand::thread_rng();
            moves.shuffle(&mut rng);

            let side = self.side_to_move();
            let opponent = if side == WHITE { BLACK } else { WHITE };
            let attacks = self.occupied[opponent];

            let piece_value = |p: Option<Piece>| match p {
                None => 0,
                Some(Piece::Pawn) => 10,
                Some(Piece::Knight) => 30,
                Some(Piece::Bishop) => 32,
                Some(Piece::Rook) => 50,
                Some(Piece::Queen) => 90,
                Some(Piece::King) => 50,
            };

            let hash_move = self.transpositions.get_move(self.key());

            moves.sort_by(|x, y| {
                match hash_move {
                    Some(m) if x == &m => std::cmp::Ordering::Less,
                    Some(m) if y == &m => std::cmp::Ordering::Greater,
                    _ => {
                        let x_target_mask = 1u64 << x.get_to();
                        let y_target_mask = 1u64 << y.get_to();
                        match x_target_mask & attacks == 0 {
                            true => match y_target_mask & attacks == 0 {
                                true => {
                                    let x_source_mask = 1u64 << x.get_from();
                                    let y_source_mask = 1u64 << y.get_from();
                                    let x_attacker_value = piece_value(self.check_piece(side, x_source_mask));
                                    let x_defender_value = piece_value(self.check_piece(opponent, x_target_mask));
                                    let y_attacker_value = piece_value(self.check_piece(side, y_source_mask));
                                    let y_defender_value = piece_value(self.check_piece(opponent, y_target_mask));
                                    let x_score = x_attacker_value - x_defender_value;
                                    let y_score = y_attacker_value - y_defender_value;
                                    x_score.cmp(&y_score)
                                },
                                false => std::cmp::Ordering::Less,
                            },
                            false => match y_target_mask & attacks == 0 {
                                true => std::cmp::Ordering::Greater,
                                false => std::cmp::Ordering::Equal,
                            }
                        }
                    }
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

        fn negamax(&mut self, context: &mut SearchContext, depth: usize, mut alpha: i64, beta: i64) -> i64 {
            if context.time_hit {
                return 0;
            }

            if context.start_time.elapsed().unwrap().as_millis() >= context.target_time {
                context.time_hit = true;
                return 0;
            }

            let side = self.side_to_move();

            if self.triple_repetition() || self.half_moves_clock >= 100 {
                return 0;
            }

            let key = self.key();
            match self.transpositions.get(key, depth, alpha, beta) {
                None => (),
                Some((score, m)) => {
                    if depth == context.depth {
                        context.best_move = m;
                    }
                    return score;
                },
            }

            if depth == 0 {
                return self.qsearch(context, 0, alpha, beta);
            }

            context.nodes += 1;
            let mut moves = self.generate_moves(false);

            if moves.is_empty() {
                return if self.in_check(side) {
                    -(10000 - (context.depth as i64 - depth as i64)) // checkmate in N moves
                } else {
                    0 // stalemate
                }
            }

            let mut best = NULL_MOVE;

            self.order_moves(&mut moves);

            let mut found_exact = false;

            for m in moves {
                self.make_move(m.clone());
                let key = self.key();
                let score = -self.negamax(context, depth - 1, -beta, -alpha);
                self.unmake_move();

                if score >= beta {
                    self.transpositions.set(key, depth, LowerBound(beta), m);
                    return beta;
                }

                if score > alpha {
                    best = m;
                    found_exact = true;
                    alpha = score;

                    if depth == context.depth {
                        context.best_move = m;
                    }
                }
            }

            self.transpositions.set(self.key(), depth, if found_exact { Exact(alpha) } else { UpperBound(alpha) }, best);
            context.seldepth = max(context.seldepth, context.depth - depth);

            alpha
        }

        fn qsearch(&mut self, context: &mut SearchContext, depth_in: usize, mut alpha: i64, beta: i64) -> i64 {
            if context.time_hit {
                return 0;
            }

            if context.start_time.elapsed().unwrap().as_millis() >= context.target_time {
                context.time_hit = true;
                return 0;
            }

            let side = self.side_to_move();
            let multiplier = if side == WHITE { 1 } else { -1 };

            context.nodes += 1;

            let score = match self.in_checkmate(side) {
                true => -(10000 - (context.depth as i64 + depth_in as i64)),
                false => self.eval() * multiplier,
            };

            if score >= beta {
                return beta;
            }

            if score > alpha {
                alpha = score;
            }

            let mut moves = self.generate_moves(true);
            self.order_moves(&mut moves);

            for capture in moves {
                self.make_move(capture);
                let score = -self.qsearch(context, depth_in + 1, -beta, -alpha);
                self.unmake_move();

                if score >= beta {
                    return beta;
                }
                if score > alpha {
                    alpha = score;
                }
            }

            context.seldepth = max(context.seldepth, context.depth + depth_in);

            alpha
        }
    }
}
