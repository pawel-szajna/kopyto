use crate::board::Board;
use crate::types::{Bitboard, Move};

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
    pub target_time: Option<u64>,
    pub depth: Option<i64>,
}

impl Options {
    pub fn new() -> Self {
        Self {
            white_time: u64::MAX,
            black_time: u64::MAX,
            white_increment: 0,
            black_increment: 0,
            target_time: None,
            depth: None,
        }
    }
}

pub struct MoveList {
    moves: Vec<Move>,
    weights: Vec<i64>,
    used: usize,
}

impl MoveList {
    fn new(moves: Vec<Move>, weights: Vec<i64>) -> Self {
        assert_eq!(moves.len(), weights.len());
        Self {
            moves,
            weights,
            used: 0,
        }
    }
}

impl Iterator for MoveList {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.used >= self.moves.len() {
            return None;
        }

        let mut max_value = i64::MIN;
        let mut max_idx = 0;
        for i in self.used..self.moves.len() {
            let weight = self.weights[i];
            if weight > max_value {
                max_idx = i;
                max_value = weight;
            }
        }

        let best_move = self.moves[max_idx];
        self.weights.swap(self.used, max_idx);
        self.moves.swap(self.used, max_idx);
        self.used += 1;

        Some(best_move)
    }
}

pub trait Search: pimpl::SearchImpl {
    fn search(&mut self, options: Options) -> Move {
        self.search_impl(options)
    }
}

impl Search for Board {}

mod pimpl {
    use std::cmp::{min, max};
    use std::thread;
    use std::time::{Duration, SystemTime};
    use super::*;
    use crate::types::{Piece, Side};
    use crate::moves_generation::MoveGenerator;
    use rand::prelude::SliceRandom;
    use rand::Rng;
    use crate::transpositions::Score;
    use crate::util;

    const NULL_MOVE: Move = Move::new();
    const MIN_MOVE_TIME: u128 = 200;
    const MAX_DEPTH: i64 = 100;

    pub struct SearchContext {
        depth: i64,
        seldepth: i64,
        best_move: Move,
        nodes: u64,
        clock_queries: usize,
        start_time: SystemTime,
        target_time: u128,
        time_hit: bool,
    }

    impl SearchContext {
        pub fn new(depth: i64, start_time: SystemTime, target_time: u128) -> Self {
            Self {
                depth,
                seldepth: 0,
                best_move: NULL_MOVE,
                nodes: 0,
                clock_queries: 0,
                start_time,
                target_time,
                time_hit: false,
            }
        }
    }

    pub trait SearchImpl {
        fn search_impl(&mut self, options: Options) -> Move;

        fn get_moves(&mut self, captures_only: bool) -> MoveList;
        fn order_moves(&self, moves: &Vec<Move>) -> Vec<i64>;

        fn eval(&self) -> i64;
        fn eval_piece(&self, mask: Bitboard, value: i64, weights: &[i64; 64]) -> i64;

        fn bishop_pair(&self, side: Side) -> bool;
        fn insufficient_material(&self) -> bool;
        fn draw_conditions(&self) -> bool;
        fn break_conditions(&mut self, context: &mut SearchContext, depth: i64, alpha: i64, beta: i64) -> Option<i64>;
        fn no_moves_conditions(&mut self, context: &SearchContext, depth: i64, moves: &MoveList) -> Option<i64>;

        fn negamax(&mut self, context: &mut SearchContext, depth: i64, alpha: i64, beta: i64) -> i64;
        fn zero_window(&mut self, context: &mut SearchContext, depth: i64, beta: i64) -> i64;
        fn qsearch(&mut self, context: &mut SearchContext, depth: i64, alpha: i64, beta: i64) -> i64;
    }

    fn get_pv(board: &mut Board, limit: i64) -> String {
        if limit <= 0 {
            return String::new();
        }

        let moves = board.generate_moves(false);
        let best = board.transpositions.get_move(board.key());

        match best {
            Some(m) if moves.contains(&m) => {
                board.make_move(m);
                let result = format!(" {:?}{}", m, get_pv(board, limit - 1));
                board.unmake_move();
                result
            },
            _ => String::new(),
        }
    }

    fn print_search_info(board: &mut Board, context: &SearchContext, current_depth: i64, score: i64) {
        let time = context.start_time.elapsed().unwrap();
        let pv = get_pv(board, max(current_depth, context.seldepth));
        println!(
            "info depth {} seldepth {} score {} nodes {} nps {} time {} hashfull {} pv{}",
            current_depth,
            context.seldepth,
            util::eval_to_str(score),
            context.nodes,
            1000000000 * context.nodes as u128 / max(1, time.as_nanos()),
            time.as_millis(),
            board.transpositions.usage(),
            if pv.is_empty() { String::from(" ?") } else { pv },
        );
    }

    fn out_of_time(context: &mut SearchContext) -> bool {
        if context.time_hit {
            return true;
        }

        // profiler actually said that this was quite costly, but since we are processing
        // millions of nodes per second, checking the clock once every 1000th is probably
        // acceptable
        context.clock_queries += 1;
        if context.clock_queries > 1000 {
            context.clock_queries = 0;
            if context.start_time.elapsed().unwrap().as_millis() >= context.target_time {
                context.time_hit = true;
                return true;
            }
        }

        false
    }

    impl SearchImpl for Board {
        fn search_impl(&mut self, options: Options) -> Move {
            let start = SystemTime::now();

            let side = self.side_to_move();

            if let Some(book_moves) = self.book.search(side, self.full_moves_count, self.key()) {
                let mut legal_moves = self.generate_moves(false);
                legal_moves.retain(|m| book_moves.contains(m));
                if !legal_moves.is_empty() {
                    let mut rng = rand::thread_rng();
                    if let Some(m) = legal_moves.choose(&mut rng) {
                        thread::sleep(Duration::from_millis(rng.gen_range(50..100)));
                        println!("info depth 1 score cp 0");
                        return m.clone();
                    }
                }
            }

            let our_time = side.choose(options.white_time, options.black_time);
            let opponent_time = !side.choose(options.white_time, options.black_time);
            let time_advantage = our_time as i64 - opponent_time as i64;
            let time_advantage_modifier = if time_advantage > 0 { time_advantage / 4 } else { time_advantage / 8 };

            let target_time = match options.target_time {
                Some(target_time) => target_time - 100,
                None => {
                    let divider = match self.full_moves_count {
                        m if m < 2 => 60,
                        m if m < 4 => 25,
                        m if m < 6 => 12,
                        _ => 8,
                    };
                    our_time / divider + max(0, time_advantage_modifier) as u64
                },
            };

            println!("info string our_time {} opponent_time {} time_advantage {} advantage_modifier {} target_time {}", our_time ,opponent_time, time_advantage, time_advantage_modifier, target_time);

            let depth = min(options.depth.unwrap_or(i64::MAX), MAX_DEPTH);
            let mut context = SearchContext::new(depth, start, max(target_time as u128, MIN_MOVE_TIME));
            let mut eval = self.last_eval;
            let mut abs_eval = 0;
            let mut best_move = NULL_MOVE;

            for current_depth in 1..=depth {
                context.depth = current_depth;
                let iter_start = SystemTime::now();
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

                best_move = context.best_move;

                abs_eval = side.choose(eval, -eval);

                let time_taken = context.start_time.elapsed().unwrap();
                let iter_taken = iter_start.elapsed().unwrap();

                print_search_info(self, &context, context.depth, abs_eval);

                if time_taken.as_millis() >= context.target_time || iter_taken.as_millis() > context.target_time / 8 {
                    break;
                }
            }

            if best_move == NULL_MOVE {
                println!("info string null move selected as best, bug? overriding with a semi-random legal move");
                let legal_moves = self.generate_moves(false);
                if !legal_moves.is_empty() {
                    best_move = legal_moves[0];
                } else {
                    println!("info string no legal moves?!");
                }
                println!("info string current position is {}", self.export_fen());
            }

            if context.time_hit {
                print_search_info(self, &context, context.depth - 1, abs_eval);
            }

            self.last_eval = -eval;
            best_move
        }

        fn get_moves(&mut self, captures_only: bool) -> MoveList {
            let moves = self.generate_moves(captures_only);
            let weights = self.order_moves(&moves);
            MoveList::new(moves, weights)
        }

        fn order_moves(&self, moves: &Vec<Move>) -> Vec<i64> {
            let side = self.side_to_move();
            let opponent = !side;
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

            let mut verified = [false; 64];
            let mut pieces = [None; 64];

            for m in &*moves {
                let source = m.get_from();
                let target = m.get_to();
                if !verified[source] {
                    pieces[source] = self.check_piece(side, Bitboard::from(source));
                    verified[source] = true;
                }
                if !verified[target] {
                    pieces[target] = self.check_piece(opponent, Bitboard::from(target));
                    verified[target] = true;
                }
            }

            moves.iter().map(|m| {
                match hash_move {
                    Some(hashed) if &hashed == m => 10000,
                    _ => {
                        let target_mask = Bitboard::from(m.get_to());
                        match (target_mask & attacks).not_empty() {
                            false => 0,
                            true => {
                                let defender_value = piece_value(pieces[m.get_to() as usize]);
                                let attacker_value = piece_value(pieces[m.get_from() as usize]);
                                defender_value * 10 - attacker_value
                            }
                        }
                    },
                }
            }).collect()
        }

        fn eval(&self) -> i64 {
            let mut score = 0i64;

            for (side, modifier) in [(Side::White, 1i64), (Side::Black, -1i64)] {
                score += modifier * self.eval_piece(self.kings[side], 0, &weights::KING[side]);
                score += modifier * self.eval_piece(self.pawns[side], 100, &weights::PAWN[side]);
                score += modifier * self.eval_piece(self.knights[side], 300, &weights::KNIGHT[side]);
                score += modifier * self.eval_piece(self.bishops[side], 320, &weights::BISHOP[side]);
                score += modifier * self.eval_piece(self.rooks[side], 500, &weights::ROOK[side]);
                score += modifier * self.eval_piece(self.queens[side], 900, &weights::QUEEN[side]);
            }

            score
        }

        fn eval_piece(&self, mask: Bitboard, value: i64, weights: &[i64; 64]) -> i64 {
            let mut score = 0;
            for pos in mask {
                score += value + weights[pos];
            }
            score
        }

        fn bishop_pair(&self, side: Side) -> bool {
            let bishops = self.bishops[side];
            let mut lsb = 0;
            let mut dsb = 0;
            for bishop in bishops {
                match bishop.is_white() {
                    true => lsb += 1,
                    false => dsb += 1,
                }
            }
            lsb > 0 && dsb > 0
        }

        fn insufficient_material(&self) -> bool {
            !(
                self.queens[Side::White].not_empty() ||
                self.queens[Side::Black].not_empty() ||
                self.rooks[Side::White].not_empty() ||
                self.rooks[Side::Black].not_empty() ||
                self.pawns[Side::White].not_empty() ||
                self.pawns[Side::Black].not_empty() ||
                self.knights[Side::White].pieces() > 3 ||
                self.knights[Side::Black].pieces() > 3 ||
                (self.bishops[Side::White].not_empty() && self.knights[Side::White].not_empty()) ||
                (self.bishops[Side::Black].not_empty() && self.bishops[Side::Black].not_empty()) ||
                self.bishop_pair(Side::White) ||
                self.bishop_pair(Side::Black)
            )
        }

        fn draw_conditions(&self) -> bool {
            self.repeated_position() || self.half_moves_clock >= 100 || self.insufficient_material()
        }

        fn break_conditions(&mut self, context: &mut SearchContext, depth: i64, alpha: i64, beta: i64) -> Option<i64> {
            if depth == context.depth {
                return None; // do not exit early from search root
            }

            if out_of_time(context) {
                return Some(0);
            }

            if self.draw_conditions() {
                return Some(0);
            }

            if let Some((score, _)) = self.transpositions.get(self.key(), depth, alpha, beta) {
                return Some(score);
            }

            None
        }

        fn no_moves_conditions(&mut self, context: &SearchContext, depth: i64, moves: &MoveList) -> Option<i64> {
            match moves.moves.is_empty() {
                false => None,
                true => Some(match self.in_check(self.side_to_move()) {
                    false => 0, // stalemate
                    true => -(10000 - (context.depth - depth)), // checkmate in N
                })
            }
        }

        fn negamax(&mut self, context: &mut SearchContext, depth: i64, mut alpha: i64, beta: i64) -> i64 {
            if let Some(score) = self.break_conditions(context, depth, alpha, beta) {
                return score;
            }

            if depth == 0 {
                return self.qsearch(context, 0, alpha, beta);
            }

            context.nodes += 1;
            let moves = self.get_moves(false);

            if let Some(score) = self.no_moves_conditions(context, depth, &moves) {
                return score;
            }

            let mut best = NULL_MOVE;
            let mut found_exact = false;

            for m in moves {
                self.make_move(m.clone());
                let key = self.key();
                let score = match found_exact {
                    false => -self.negamax(context, depth - 1, -beta, -alpha),
                    true => {
                        let mut score = -self.zero_window(context, depth - 1, -alpha);
                        if score > alpha {
                            score = -self.negamax(context, depth - 1, -beta, -alpha);
                        }
                        score
                    }
                };
                self.unmake_move();

                if context.time_hit {
                    return 0;
                }

                if score >= beta {
                    self.transpositions.set(key, depth, Score::LowerBound(beta), m);
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

            self.transpositions.set(self.key(), depth, Score::from_alpha(alpha, found_exact), best);
            context.seldepth = max(context.seldepth, context.depth - depth);

            alpha
        }

        fn zero_window(&mut self, context: &mut SearchContext, depth: i64, beta: i64) -> i64 {
            if let Some(score) = self.break_conditions(context, depth, beta - 1, beta) {
                return score;
            }

            if depth == 0 {
                return self.qsearch(context, 0, beta - 1, beta);
            }

            context.nodes += 1;
            let moves = self.get_moves(false);

            if let Some(score) = self.no_moves_conditions(context, depth, &moves) {
                return score;
            }

            for m in moves {
                self.make_move(m);
                let eval = -self.zero_window(context, depth - 1, 1 - beta);
                self.unmake_move();

                if eval >= beta {
                    return beta;
                }
            }

            beta - 1
        }

        fn qsearch(&mut self, context: &mut SearchContext, depth: i64, mut alpha: i64, beta: i64) -> i64 {
            if let Some(score) = self.break_conditions(context, depth, alpha, beta) {
                return score;
            }

            let side = self.side_to_move();
            let multiplier = side.choose(1, -1);

            context.nodes += 1;

            let score = self.eval() * multiplier;

            if score >= beta {
                return beta;
            }

            if score > alpha {
                alpha = score;
            }

            let moves = self.get_moves(true);
            let mut best = NULL_MOVE;
            let mut found_exact = false;

            for capture in moves {
                self.make_move(capture);
                let score = -self.qsearch(context, depth - 1, -beta, -alpha);
                self.unmake_move();

                if context.time_hit {
                    return 0;
                }

                if score >= beta {
                    self.transpositions.set(self.key(), depth, Score::LowerBound(beta), capture);
                    return beta;
                }

                if score > alpha {
                    alpha = score;
                    best = capture;
                    found_exact = true;
                }
            }

            if best != NULL_MOVE {
                self.transpositions.set(self.key(), depth, Score::from_alpha(alpha, found_exact), best);
                context.seldepth = max(context.seldepth, context.depth - depth);
            }

            alpha
        }
    }
}
