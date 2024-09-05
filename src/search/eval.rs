use crate::board::Board;
use crate::types::{Bitboard, Side};

pub type Score = i16;

mod weights {
    use super::*;

    pub type Weights = [Score; 64];
    pub type WeightsPerSide = [Weights; 2];

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

pub fn evaluate(board: &Board) -> Score {
    let mut score = 0;

    for (side, modifier) in [(Side::White, 1), (Side::Black, -1)] {
        score += modifier * eval_piece(board.kings[side], 0, &weights::KING[side]);
        score += modifier * eval_piece(board.pawns[side], 100, &weights::PAWN[side]);
        score += modifier * eval_piece(board.knights[side], 300, &weights::KNIGHT[side]);
        score += modifier * eval_piece(board.bishops[side], 320, &weights::BISHOP[side]);
        score += modifier * eval_piece(board.rooks[side], 500, &weights::ROOK[side]);
        score += modifier * eval_piece(board.queens[side], 900, &weights::QUEEN[side]);
    }

    score
}

fn eval_piece(mask: Bitboard, value: Score, weights: &weights::Weights) -> Score {
    let mut score = 0;
    for pos in mask {
        score += value + weights[pos];
    }
    score
}
