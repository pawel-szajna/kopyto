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
use crate::search::{book, eval, Options};
use crate::transpositions::{TableScore, Transpositions};
use crate::types::{Bitboard, Move};

const NULL_MOVE: Move = Move::new();
const MAX_DEPTH: i16 = 64;
pub const KILLER_MOVES_STORED: usize = 3;

const ALL_MOVES: bool = false;
const CAPTURES_ONLY: bool = true;

pub struct Searcher {
    board: Board,
    transpositions: Transpositions,

    depth: i16,
    seldepth: i16,

    last_eval: Score,
    best_move: Move,
    killers: [[Move; KILLER_MOVES_STORED]; MAX_DEPTH as usize],
    history: [[[u32; 64]; 64]; 2],

    nodes: u64,

    clock_queries: usize,
    start_time: SystemTime,
    target_time: u128,
    time_hit: bool,
}

impl Searcher {
    pub fn new(board: Board) -> Self {
        Self {
            board,
            transpositions: Transpositions::new(),

            depth: 0,
            seldepth: 0,

            last_eval: 0,
            best_move: NULL_MOVE,
            killers: [[NULL_MOVE; KILLER_MOVES_STORED]; MAX_DEPTH as usize],
            history: [[[0; 64]; 64]; 2],

            nodes: 0,

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

    fn print_search_info(&mut self, current_depth: i16, score: Score, pv: &str) {
        let time = self.start_time.elapsed().unwrap();
        println!(
            "info depth {} seldepth {} score {} nodes {} nps {} time {} hashfull {} pv{}",
            current_depth,
            self.seldepth,
            if score.abs() > 9000 {
                format!("mate {}", score.signum() * (1 + (10000 - score.abs())) / 2)
            } else {
                format!("cp {}", score)
            },
            self.nodes,
            1000000000 * self.nodes as u128 / max(1, time.as_nanos()),
            time.as_millis(),
            self.transpositions.usage(),
            pv,
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
        // return None;
        // TODO: book should be opt-in, not default
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
            return Some(score);
        }

        None
    }

    fn no_moves_conditions(&mut self, depth: i16, moves: &MoveList) -> Option<Score> {
        match moves.is_empty() {
            false => None,
            true => Some(match self.board.in_check(self.board.side_to_move()) {
                false => 0, // stalemate
                true => -(10000 - (self.depth - depth)), // checkmate in N
            })
        }
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

        for current_depth in 1..=target_depth {
            let iter_start = SystemTime::now();
            let last_eval = eval;

            self.depth = current_depth;
            self.seldepth = 0;

            let window_size = 40;

            eval = self.negamax(current_depth, last_eval - window_size, last_eval + window_size);
            if self.time_hit {
                break;
            }

            if (last_eval - eval).abs() >= window_size {
                eval = self.negamax(current_depth, Score::MIN + 1, Score::MAX);
                if self.time_hit {
                    break;
                }
            }

            best_move = self.best_move;
            abs_eval = self.board.current_color.choose(eval, -eval);
            pv = self.get_pv(current_depth);

            let time_taken = self.start_time.elapsed().unwrap();
            let iter_taken = iter_start.elapsed().unwrap();

            self.print_search_info(self.depth, abs_eval, pv.as_str());

            if time_taken.as_millis() >= self.target_time || iter_taken.as_millis() > self.target_time / 8 {
                break;
            }
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
            self.print_search_info(self.depth - 1, abs_eval, pv.as_str());
        }

        self.last_eval = -eval;
        best_move
    }

    fn negamax(&mut self, depth: i16, mut alpha: Score, beta: Score) -> Score {
        if depth == 0 {
            return self.qsearch(0, alpha, beta);
        }

        if let Some(score) = self.break_conditions(depth, alpha, beta) {
            return score;
        }

        self.nodes += 1;
        let moves = self.get_moves::<ALL_MOVES>(depth);

        if let Some(score) = self.no_moves_conditions(depth, &moves) {
            return score;
        }

        let mut best = NULL_MOVE;
        let mut found_exact = false;

        for m in moves {
            self.board.make_move(m.clone());
            let key = self.board.key();
            let score = match found_exact {
                false => -self.negamax(depth - 1, -beta, -alpha),
                true => {
                    let mut score = -self.zero_window(depth - 1, -alpha);
                    if score > alpha {
                        score = -self.negamax(depth - 1, -beta, -alpha);
                    }
                    score
                }
            };
            self.board.unmake_move();

            if self.time_hit {
                return 0;
            }

            if score >= beta {
                self.transpositions.set(key, depth, TableScore::LowerBound(beta), m);
                self.store_killer(depth, m);
                return beta;
            }

            if score > alpha {
                best = m;
                found_exact = true;
                alpha = score;

                if depth == self.depth {
                    self.best_move = m;
                }
            }
        }

        self.transpositions.set(self.board.key(), depth, TableScore::from_alpha(alpha, found_exact), best);
        self.seldepth = max(self.seldepth, self.depth - depth);

        alpha
    }

    fn zero_window(&mut self, depth: i16, beta: Score) -> Score {
        if depth == 0 {
            return self.qsearch(0, beta - 1, beta);
        }

        if let Some(score) = self.break_conditions(depth, beta - 1, beta) {
            return score;
        }

        self.nodes += 1;
        let moves = self.get_moves::<ALL_MOVES>(depth);

        if let Some(score) = self.no_moves_conditions(depth, &moves) {
            return score;
        }

        for m in moves {
            self.board.make_move(m);
            let eval = -self.zero_window(depth - 1, 1 - beta);
            self.board.unmake_move();

            if eval >= beta {
                self.store_killer(depth, m);
                return beta;
            }
        }

        beta - 1
    }

    fn qsearch(&mut self, depth: i16, mut alpha: Score, beta: Score) -> Score {
        if let Some(score) = self.break_conditions(depth, alpha, beta) {
            return score;
        }

        let side = self.board.side_to_move();
        let multiplier = side.choose(1, -1);

        self.nodes += 1;

        let score = eval::evaluate(&self.board, Verbosity::Quiet) * multiplier;

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
            let score = -self.qsearch(depth - 1, -beta, -alpha);
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
