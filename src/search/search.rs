use std::cmp::{max, min};
use std::thread;
use std::time::{Duration, SystemTime};
use rand::Rng;
use rand::seq::SliceRandom;
use crate::board::{Board, FenProducer};
use crate::moves_generation;
use crate::moves_generation::MoveList;
use crate::search::checks::Checks;
use crate::search::eval::{Score, Verbosity};
use crate::search::{book, eval, weights, Options};
use crate::transpositions::{TableScore, Transpositions};
use crate::types::{Bitboard, Move, Piece, Side};

const NULL_MOVE: Move = Move::new();
const MAX_DEPTH: i16 = 64;
pub const KILLER_MOVES_STORED: usize = 3;

const ALL_MOVES: bool = false;
const CAPTURES_ONLY: bool = true;

pub struct Searcher<'a> {
    board: Board,
    transpositions: &'a mut Transpositions,

    book: bool,

    depth: i16,
    seldepth: i16,

    last_eval: Score,
    best_move: Move,
    killers: [[Move; KILLER_MOVES_STORED]; MAX_DEPTH as usize],
    history: [[[u32; 64]; 64]; 2],

    nodes: u64,
    tbhits: u64,
    nodes_n: u64,
    nodes_z: u64,
    nodes_q: u64,
    delta_prunes: u64,
    razoring_attempts: u64,
    razoring_success: u64,

    clock_queries: usize,
    start_time: SystemTime,
    target_time: u128,
    time_hit: bool,
}

impl<'a> Searcher<'a> {
    pub fn new(board: Board, transpositions: &'a mut Transpositions, book: bool) -> Self {
        Self {
            board,
            transpositions,

            book,

            depth: 0,
            seldepth: 0,

            last_eval: 0,
            best_move: NULL_MOVE,
            killers: [[NULL_MOVE; KILLER_MOVES_STORED]; MAX_DEPTH as usize],
            history: [[[0; 64]; 64]; 2],

            nodes: 0,
            tbhits: 0,
            nodes_n: 0,
            nodes_z: 0,
            nodes_q: 0,
            delta_prunes: 0,
            razoring_attempts: 0,
            razoring_success: 0,

            clock_queries: 0,
            start_time: SystemTime::now(),
            target_time: 0,
            time_hit: false,
        }
    }

    fn get_pv(&mut self, limit: i16) -> String {
        if limit <= 0 {
            return String::new();
        }

        let moves = moves_generation::generate_all(&self.board);
        let best = self.transpositions.get_move(self.board.key());

        match best {
            Some(m) if moves.contains(&m) => {
                self.board.make_move(m);
                let result = format!(" {:?}{}", m, self.get_pv(limit - 1));
                self.board.unmake_move();
                result
            },
            _ => String::new(),
        }
    }

    fn print_search_info(&mut self, current_depth: i16, score: Score, pv: &str, aspiration_fail: bool) {
        let time = self.start_time.elapsed().unwrap();
        println!(
            "info depth {} seldepth {} score {} nodes {} nps {} time {} hashfull {} tbhits {} pv{} string nodes_n {} nodes_z {} nodes_q {} dprunes {} asp_retry {} razor att {} succ {}",
            current_depth,
            self.seldepth.max(current_depth),
            if score.abs() > 9000 {
                format!("mate {}", score.signum() * (1 + (10000 - score.abs())) / 2)
            } else {
                format!("cp {}", score * match self.board.side_to_move() {
                    Side::White => 1,
                    Side::Black => -1,
                })
            },
            self.nodes,
            1000000000 * self.nodes as u128 / max(1, time.as_nanos()),
            time.as_millis(),
            self.transpositions.usage(),
            self.tbhits,
            pv,
            self.nodes_n,
            self.nodes_z,
            self.nodes_q,
            self.delta_prunes,
            aspiration_fail,
            self.razoring_attempts,
            self.razoring_success,
        );
    }

    fn out_of_time(&mut self) -> bool {
        if self.time_hit {
            return true;
        }

        // profiler actually said that this was quite costly, but since we are processing
        // millions of nodes per second, checking the clock once every 1000th is probably
        // acceptable
        self.clock_queries += 1;
        if self.clock_queries > 1000 {
            self.clock_queries = 0;
            if self.start_time.elapsed().unwrap().as_millis() >= self.target_time {
                self.time_hit = true;
                return true;
            }
        }

        false
    }

    fn calculate_target_time(&self, options: &Options) -> u128 {
        if let Some(requested_time) = options.target_time {
            return requested_time as u128 - 100;
        }

        let side = self.board.side_to_move();
        let our_time = side.choose(options.white_time, options.black_time);
        let opponent_time = (!side).choose(options.white_time, options.black_time);
        let time_advantage = our_time - opponent_time;
        let time_advantage_modifier = if time_advantage > 0 { time_advantage / 4 } else { time_advantage / 8 };
        let divider = match self.board.full_moves_count {
            m if m < 2 => 60,
            m if m < 4 => 25,
            m if m < 6 => 12,
            _ => 8,
        };

        let result = (our_time / divider + max(0, time_advantage_modifier)) as u128;

        println!(
            "info string our time: {} opponent time: {} time advantage: {} advantage modifier: {} moves count: {} divider: {} target time: {}",
            our_time, opponent_time, time_advantage, time_advantage_modifier, self.board.full_moves_count, divider, result);

        result
    }

    fn get_book_move(&self) -> Option<Move> {
        if !self.book {
            return None;
        }

        if let Some(book_moves) = book::BOOK.search(self.board.current_color, self.board.full_moves_count, self.board.key()) {
            let mut legal_moves = moves_generation::generate_all(&self.board);
            legal_moves.retain(|m| book_moves.contains(m));
            if !legal_moves.is_empty() {
                let mut rng = rand::thread_rng();
                if let Some(m) = legal_moves.choose(&mut rng) {
                    thread::sleep(Duration::from_millis(rng.gen_range(50..100)));
                    println!("info depth 1 score cp 0");
                    return Some(m.clone());
                }
            }
        }

        None
    }

    fn get_moves<const CAPTURES_ONLY: bool>(&mut self, depth: i16) -> MoveList {
        let moves = if CAPTURES_ONLY {
            moves_generation::generate_captures(&self.board)
        } else {
            moves_generation::generate_all(&self.board)
        };
        let weights = moves_generation::order(
            &self.board,
            &moves,
            self.transpositions.get_move(self.board.key()),
            &self.killers[if depth > 0 { depth } else { MAX_DEPTH - 1 } as usize],
            &self.history[self.board.side_to_move()]);
        MoveList::new(moves, weights)
    }

    fn break_conditions(&mut self, depth: i16, alpha: Score, beta: Score) -> Option<Score> {
        if depth == self.depth {
            return None; // do not exit early from search root
        }

        if self.out_of_time() {
            return Some(0);
        }

        if self.board.draw_conditions() {
            return Some(0);
        }

        if let Some(score) = self.transpositions.get(self.board.key(), depth, alpha, beta) {
            self.tbhits += 1;
            return Some(score);
        }

        None
    }

    fn checkmate_score(&self, ply: i16) -> Score {
        -(10000 - ply)
    }

    fn no_moves_conditions(&mut self, ply: i16, moves: &MoveList) -> Option<Score> {
        match moves.is_empty() {
            false => None,
            true => Some(match self.board.in_check() {
                false => 0, // stalemate
                true => self.checkmate_score(ply), // checkmate in N
            })
        }
    }

    fn late_move_reduction(&mut self, depth: i16, m: Move, move_counter: i32) -> i16 {
        let depth_from_root = self.depth - depth;
        if depth_from_root > 3 && move_counter > 4
            && !self.killers[depth as usize].contains(&m)
            && !self.board.in_check() {
            return if move_counter < 12 { 1 } else { 2 };
        }
        0
    }

    fn store_killer(&mut self, depth: i16, m: Move) {
        if (self.board.any_piece & Bitboard::from(m.get_to())).not_empty() {
            return;
        }

        let depth = depth as usize;

        let side = self.board.side_to_move();
        let from = m.get_from();
        let to = m.get_to();
        let history_bonus = (depth as u32 * depth as u32).clamp(1, 16384);
        let current_history = self.history[side][from][to];
        let history_penalty = (current_history * history_bonus) / 16384;
        self.history[side][from][to] += history_bonus - history_penalty;

        if self.killers[depth].contains(&m) {
            return;
        }

        self.killers[depth].rotate_right(1);
        self.killers[depth][0] = m;
    }

    fn mate_distance_pruning(&self, ply: i16, alpha: &mut Score, beta: &mut Score) -> Option<Score> {
        let new_alpha = (*alpha).max(self.checkmate_score(ply));
        let new_beta = (*beta).min(-self.checkmate_score(ply));

        if new_alpha >= new_beta {
            return Some(new_alpha)
        }

        *alpha = new_alpha;
        *beta = new_beta;

        None
    }

    pub fn go(&mut self, options: Options) -> Move {
        if let Some(book_move) = self.get_book_move() {
            return book_move;
        }

        let target_depth = min(options.depth.unwrap_or(i16::MAX), MAX_DEPTH - 1);
        self.start_time = SystemTime::now();
        self.target_time = self.calculate_target_time(&options);

        let mut eval = self.last_eval;
        let mut abs_eval = 0;
        let mut best_move = NULL_MOVE;
        let mut pv = String::new();

        let mut consecutive_evals = 0;
        let mut last_turn = eval;
        let mut last_move = NULL_MOVE;

        for current_depth in 1..=target_depth {
            let iter_start = SystemTime::now();
            let last_eval = eval;

            self.depth = current_depth;
            self.seldepth = 0;

            let window_size = 40;
            let mut aspiration_fail = false;

            eval = self.negamax(0, current_depth, last_eval - window_size, last_eval + window_size, true);
            if self.time_hit {
                break;
            }

            if (last_eval - eval).abs() >= window_size {
                aspiration_fail = true;
                eval = self.negamax(0, current_depth, Score::MIN + 1, Score::MAX, true);
                if self.time_hit {
                    break;
                }
            }

            best_move = self.best_move;
            abs_eval = self.board.current_color.choose(eval, -eval);
            pv = self.get_pv(current_depth);

            let time_taken = self.start_time.elapsed().unwrap();
            let iter_taken = iter_start.elapsed().unwrap();

            self.print_search_info(self.depth, abs_eval, pv.as_str(), aspiration_fail);

            if time_taken.as_millis() >= self.target_time || iter_taken.as_millis() > self.target_time / 8 {
                break;
            }

            if best_move == last_move && abs_eval == last_turn {
                consecutive_evals += 1;
            } else {
                consecutive_evals = 0;
            }

            if consecutive_evals > 8 && abs_eval.abs() >= self.checkmate_score(MAX_DEPTH).abs() {
                break;
            }

            last_move = best_move;
            last_turn = abs_eval;
        }

        if best_move == NULL_MOVE {
            println!("info string null move selected as best, bug? overriding with a semi-random legal move");
            let legal_moves = moves_generation::generate_all(&self.board);
            if !legal_moves.is_empty() {
                best_move = legal_moves[0];
            } else {
                println!("info string no legal moves?!");
            }
            println!("info string current position is {}", self.board.export_fen());
        }

        if self.time_hit {
            self.print_search_info(self.depth - 1, abs_eval, pv.as_str(), false);
        }

        self.last_eval = -eval;
        best_move
    }

    fn negamax(&mut self, ply: i16, depth: i16, mut alpha: Score, mut beta: Score, root: bool) -> Score {
        if depth <= 0 {
            return self.qsearch(ply, 0, alpha, beta);
        }

        if let Some(score) = self.break_conditions(depth, alpha, beta) {
            return score;
        }

        if let Some(score) = self.mate_distance_pruning(ply, &mut alpha, &mut beta) {
            return score;
        }

        self.nodes += 1;
        self.nodes_n += 1;
        let moves = self.get_moves::<ALL_MOVES>(depth);

        if let Some(score) = self.no_moves_conditions(ply, &moves) {
            return score;
        }

        let mut best = NULL_MOVE;
        let mut found_exact = false;
        let mut move_counter = 0;

        #[cfg(feature = "check_extension")] // This is fucking ridiculous
        let mut depth = depth;
        #[cfg(feature = "check_extension")]
        if self.board.in_check() {
            depth += 1;
        }

        for m in moves {
            self.board.make_move(m.clone());

            let score = match move_counter > 0 {
                false => -self.negamax(ply + 1, depth - 1, -beta, -alpha, false),
                true => {
                    let mut next_depth = depth - 1;
                    next_depth -= self.late_move_reduction(depth, m, move_counter);

                    let mut score = -self.zero_window(ply + 1, next_depth, -alpha, false);
                    if score > alpha {
                        score = -self.negamax(ply + 1, depth - 1, -beta, -alpha, false);
                    }
                    score
                }
            };

            self.board.unmake_move();

            if self.time_hit {
                return 0;
            }

            if score >= beta {
                self.transpositions.set(self.board.key(), depth, TableScore::LowerBound(beta), m);
                self.store_killer(depth, m);
                return beta;
            }

            if score > alpha {
                best = m;
                found_exact = true;
                alpha = score;

                if root {
                    self.best_move = m;
                }
            }

            move_counter += 1;
        }

        self.transpositions.set(self.board.key(), depth, TableScore::from_alpha(alpha, found_exact), best);
        self.seldepth = max(self.seldepth, self.depth - depth);

        alpha
    }

    fn zero_window(&mut self, ply: i16, mut depth: i16, mut beta: Score, last_null: bool) -> Score {
        if depth <= 0 {
            return self.qsearch(ply, 0, beta - 1, beta);
        }

        if let Some(score) = self.break_conditions(depth, beta - 1, beta) {
            return score;
        }

        if let Some(score) = self.mate_distance_pruning(ply, &mut (beta - 1), &mut beta) {
            return score;
        }

        let current_eval = eval::evaluate(&self.board, Verbosity::Quiet) * self.board.side_to_move().choose(1, -1);

        // Razoring
        if !self.board.in_check() && current_eval + 500 + 200 * depth * depth < beta - 1 {
            self.razoring_attempts += 1;
            let quiescence_eval = self.qsearch(ply, 0, beta - 1, beta);
            if quiescence_eval < beta - 1 {
                self.razoring_success += 1;
                return quiescence_eval;
            }
        }

        // Reverse futility pruning
        if !last_null && !self.board.in_check() && depth < 3 {
            let margin = match depth {
                1 => weights::BASE_SCORES[Piece::Bishop],
                2 => weights::BASE_SCORES[Piece::Rook],
                _ => 0, // should be impossible
            };

            if current_eval - margin > beta {
                return beta;
            }
        }

        // Null move pruning
        if !last_null && !self.board.in_check() && self.board.any_piece.pieces() > 8 {
            let null_reduction = 1 + depth * 2 / 3;

            self.board.make_null();
            let value = -self.zero_window(ply + 2, depth - null_reduction, 1 - beta, true);
            self.board.unmake_null();

            if value >= beta {
                return beta;
            }
        }

        self.nodes += 1;
        self.nodes_z += 1;
        let moves = self.get_moves::<ALL_MOVES>(depth);

        if let Some(score) = self.no_moves_conditions(ply, &moves) {
            return score;
        }

        // Internal iterative deepening
        if depth > 4 && self.transpositions.get_move(self.board.key()).is_none() {
            depth -= 2;
        }

        if depth <= 0 {
            return self.qsearch(ply, 0, beta - 1, beta);
        }

        // Check extension
        #[cfg(feature = "check_extension")]
        if self.board.in_check() {
            depth += 1;
        }

        let mut move_counter = 0;

        for m in moves {
            let mut next_depth = depth - 1;
            next_depth -= self.late_move_reduction(depth, m, move_counter);

            self.board.make_move(m);
            let eval = -self.zero_window(ply + 1, next_depth, 1 - beta, false);
            self.board.unmake_move();

            if eval >= beta {
                self.transpositions.set(self.board.key(), depth, TableScore::LowerBound(beta), m);
                self.store_killer(depth, m);
                return beta;
            }

            move_counter += 1;
        }

        beta - 1
    }

    fn qsearch(&mut self, ply: i16, depth: i16, mut alpha: Score, mut beta: Score) -> Score {
        if let Some(score) = self.break_conditions(depth, alpha, beta) {
            return score;
        }

        if let Some(score) = self.mate_distance_pruning(ply, &mut alpha, &mut beta) {
            return score;
        }

        let side = self.board.side_to_move();
        let multiplier = side.choose(1, -1);

        self.nodes += 1;
        self.nodes_q += 1;

        if self.board.in_checkmate() {
            return self.checkmate_score(depth);
        }

        let score = eval::evaluate(&self.board, Verbosity::Quiet) * multiplier;

        let delta_margin = weights::BASE_SCORES[Piece::Queen];

        if score + delta_margin < alpha && !self.board.in_check() {
            self.delta_prunes += 1;
            return alpha;
        }

        if score >= beta {
            return beta;
        }

        if score > alpha {
            alpha = score;
        }

        let moves = self.get_moves::<CAPTURES_ONLY>(depth);
        let mut best = NULL_MOVE;
        let mut found_exact = false;

        for capture in moves {
            self.board.make_move(capture);
            let score = -self.qsearch(ply + 1, depth - 1, -beta, -alpha);
            self.board.unmake_move();

            if self.time_hit {
                return 0;
            }

            if score >= beta {
                self.transpositions.set(self.board.key(), depth, TableScore::LowerBound(beta), capture);
                return beta;
            }

            if score > alpha {
                alpha = score;
                best = capture;
                found_exact = true;
            }
        }

        if best != NULL_MOVE {
            self.transpositions.set(self.board.key(), depth, TableScore::from_alpha(alpha, found_exact), best);
            self.seldepth = max(self.seldepth, self.depth - depth);
        }

        alpha
    }
}
