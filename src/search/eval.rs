use crate::board::{masks, Board};
use crate::moves_generation::attacks;
use crate::search::weights;
use crate::types::{Bitboard, Piece, Side};

pub type Score = i16;

const SIDE_BONUS_VALUE: Score = 12;

const fn lerp(phase: i32, a: Score, b: Score) -> Score {
    ((a as i32 * (100 - phase) + b as i32 * phase) / 100) as i16
}

pub enum Verbosity {
    Quiet,
    Verbose,
}

pub fn evaluate(board: &Board, verbosity: Verbosity) -> Score {
    match verbosity {
        Verbosity::Quiet => Evaluator::<false>::new(board).evaluate(),
        Verbosity::Verbose => Evaluator::<true>::new(board).evaluate(),
    }
}

struct Evaluator<'a, const VERBOSE: bool> {
    board: &'a Board,
    side_multiplier: Score,
    doubled_pawns: [i16; 2],
    isolated_pawns: [i16; 2],
}

fn multiplier(side: Side) -> Score {
    match side {
        Side::White => 1,
        Side::Black => -1,
    }
}

fn count_side_pawns(board: &Board, side: Side) -> [i16; 8] {
    [0, 1, 2, 3, 4, 5, 6, 7]
        .map(|file| (board.pawns[side] & masks::FILES[file]).pieces() as i16)
}

fn doubled_pawns(pawn_counts: &[i16; 8]) -> i16 {
    pawn_counts
        .iter()
        .filter(|count| **count > 1)
        .count() as i16
}

fn isolated_pawns(pawn_counts: &[i16; 8]) -> i16 {
    let guarded_pawn_counts = [0, pawn_counts[0], pawn_counts[1], pawn_counts[2], pawn_counts[3], pawn_counts[4], pawn_counts[5], pawn_counts[6], pawn_counts[7], 0];
    let mut count = 0;

    for idx in 1..=8 {
        if guarded_pawn_counts[idx - 1] == 0 && guarded_pawn_counts[idx + 1] == 0 {
            count += 1;
        }
    }

    count
}

impl<'a, const VERBOSE: bool> Evaluator<'a, VERBOSE> {
    pub fn new(board: &'a Board) -> Self {
        let file_pawn_counts = [count_side_pawns(board, Side::White), count_side_pawns(board, Side::Black)];
        Self {
            board,
            side_multiplier: multiplier(board.side_to_move()),
            doubled_pawns: [doubled_pawns(&file_pawn_counts[Side::White]), doubled_pawns(&file_pawn_counts[Side::Black])],
            isolated_pawns: [isolated_pawns(&file_pawn_counts[Side::White]), isolated_pawns(&file_pawn_counts[Side::Black])],
        }
    }

    pub fn evaluate(&mut self) -> Score {
        let score_middle = self.evaluate_middle();
        let score_end = self.evaluate_end();

        let endgame_weight = self.endgame_weight();
        let phase_score = lerp(endgame_weight, score_middle, score_end);
        let side_bonus = self.side_bonus();
        let mobility = self.mobility();

        if VERBOSE {
            println!("score_middle: {}", score_middle);
            println!("score_end: {}", score_end);
            println!("endgame_weight: {}", endgame_weight);
            println!("phase_score: {}", phase_score);
            println!("side_bonus: {}", side_bonus);
            println!("mobility: {}", mobility);
        }

        phase_score + side_bonus + mobility
    }

    fn evaluate_middle(&self) -> Score {
        let mut score = 0;
        score += self.pieces_score_middle();
        score += self.pawn_score_middle();
        score
    }

    fn evaluate_end(&self) -> Score {
        let mut score = 0;
        score += self.pieces_score_end();
        score += self.pawn_score_end();
        score
    }

    /// Estimate how much into the endgame we are. 0 is middle game, 100 is endgame
    fn endgame_weight(&self) -> i32 {
        let min_bound = 1000;
        let max_bound = 2 * weights::SIDE_STARTING_MATERIAL as i32 - min_bound;

        let pieces_white = self.non_pawn_pieces_score(Side::White);
        let pieces_black = self.non_pawn_pieces_score(Side::Black);

        let pieces = ((pieces_white + pieces_black) as i32).clamp(min_bound, max_bound);

        if VERBOSE {
            println!("endgame_weight calculation");
            println!("-- pieces_white: {}", pieces_white);
            println!("-- pieces_black: {}", pieces_black);
            println!("-- pieces: {}", pieces);
            println!("-- min_bound: {}", min_bound);
            println!("-- max_bound: {}", max_bound);
        }

        ((max_bound - pieces) * 100) / (max_bound - min_bound)
    }

    /// Tempo bonus for the side to move
    fn side_bonus(&self) -> Score {
        SIDE_BONUS_VALUE * self.side_multiplier
    }

    /// Pieces (non-pawns) base value, side-relative value
    fn non_pawn_pieces_score(&self, side: Side) -> Score {
        let mut score = 0;
        for (&source, value) in [
            (&self.board.knights[side], weights::BASE_SCORES[Piece::Knight]),
            (&self.board.bishops[side], weights::BASE_SCORES[Piece::Bishop]),
            (&self.board.rooks[side], weights::BASE_SCORES[Piece::Rook]),
            (&self.board.queens[side], weights::BASE_SCORES[Piece::Queen]),
        ] {
            score += value * source.pieces() as i16;
        }
        score
    }

    fn pieces_score(&self, weight_set: &weights::WeightSet) -> Score {
        let mut score = 0;
        for side in [Side::White, Side::Black] {
            let multiplier = multiplier(side);
            for idx in self.board.occupied[side] {
                let piece = unsafe { self.board.pieces[side][idx].unwrap_unchecked() };
                score += multiplier * (weight_set.base[piece] + weight_set[piece][side][idx]);
            }
        }
        score
    }

    /// Middle-game pieces score calculated from base pieces score and PSQT
    fn pieces_score_middle(&self) -> Score {
        if VERBOSE {
            println!("calculating pieces_score_middle");
        }
        self.pieces_score(&weights::MID_GAME)
    }

    /// Endgame pieces score calculated from base pieces score and PSQT
    fn pieces_score_end(&self) -> Score {
        if VERBOSE {
            println!("calculating pieces_score_end");
        }
        self.pieces_score(&weights::END_GAME)
    }

    fn pawn_score_middle(&self) -> Score {
        let mut score = 0;

        for side in [Side::White, Side::Black] {
            let mut side_score = 0;

            side_score -= self.doubled_pawns[side] * 5;
            side_score -= self.isolated_pawns[side] * 4;

            score += multiplier(side) * side_score;
        }

        score
    }

    fn pawn_score_end(&self) -> Score {
        let mut score = 0;

        for side in [Side::White, Side::Black] {
            let mut side_score = 0;

            side_score -= self.doubled_pawns[side] * 20;
            side_score -= self.isolated_pawns[side] * 8;

            score += multiplier(side) * side_score;
        }

        score
    }

    fn piece_mobility(&self, attacks: Bitboard, mask: Bitboard) -> Score {
        (attacks & mask).pieces() as Score
    }

    fn mobility(&self) -> Score {
        let mut score = 0;

        for side in [Side::White, Side::Black] {
            let mut side_score = 0;

            let mut good_targets = !self.board.occupied[side];
            for enemy_pawn in self.board.pawns[!side] {
                good_targets &= !attacks::pawn(!side, enemy_pawn);
            }

            for knight in self.board.knights[side] {
                side_score += 4 * self.piece_mobility(attacks::knight(knight), good_targets);
            }

            for bishop in self.board.bishops[side] {
                side_score += 2 * self.piece_mobility(attacks::bishop(bishop, self.board.any_piece), good_targets);
            }

            for rook in self.board.rooks[side] {
                side_score += 3 * self.piece_mobility(attacks::rook(rook, self.board.any_piece), good_targets);
            }

            for queen in self.board.queens[side] {
                side_score += 2 * self.piece_mobility(attacks::queen(queen, self.board.any_piece), good_targets);
            }

            score += multiplier(side) * side_score;
        }

        score
    }
}
