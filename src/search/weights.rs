use std::ops::Index;
use crate::search::Score;
use crate::types::Piece;

pub type PieceTable = [Score; 5];

const fn acquire(pt: &PieceTable, piece: Piece) -> &Score {
    match piece {
        Piece::King => &0,
        Piece::Pawn => &pt[0],
        Piece::Knight => &pt[1],
        Piece::Bishop => &pt[2],
        Piece::Rook => &pt[3],
        Piece::Queen => &pt[4],
    }
}

impl Index<Piece> for PieceTable {
    type Output = Score;

    fn index(&self, index: Piece) -> &Self::Output {
        acquire(self, index)
    }
}

pub const BASE_SCORES: PieceTable = [50, 300, 320, 500, 900];
pub const END_SCORES: PieceTable = [80, 300, 320, 500, 900];

pub const SIDE_STARTING_MATERIAL: Score =
    *acquire(&BASE_SCORES, Piece::Knight) * 2 +
    *acquire(&BASE_SCORES, Piece::Bishop) * 2 +
    *acquire(&BASE_SCORES, Piece::Rook) * 2 +
    *acquire(&BASE_SCORES, Piece::Queen);

pub type HalfWeights = [Score; 32];
pub type Weights = [Score; 64];
pub type WeightsPerSide = [Weights; 2];

pub struct WeightSet {
    pub base: PieceTable,
    pub pawn: WeightsPerSide,
    pub knight: WeightsPerSide,
    pub bishop: WeightsPerSide,
    pub rook: WeightsPerSide,
    pub queen: WeightsPerSide,
    pub king: WeightsPerSide,
}

impl Index<Piece> for WeightSet {
    type Output = WeightsPerSide;

    fn index(&self, index: Piece) -> &Self::Output {
        match index {
            Piece::Pawn => &self.pawn,
            Piece::Knight => &self.knight,
            Piece::Bishop => &self.bishop,
            Piece::Rook => &self.rook,
            Piece::Queen => &self.queen,
            Piece::King => &self.king,
        }
    }
}

const PAWN_BASE: Weights = [
     0,   0,   0,   0,   0,   0,   0,   0,
    -3,   3,  -1,  -6,   2,  -8,   5,  -4,
     2,  -6,  -3,  11,  -4,  -2,  -8,  -4,
     6,   0,  -6,   0,   5,  -1,  -6,   5,
    -2, -12,   3,  10,  20,   8,   2,  -4,
    -4,  -8,   6,   7,  16,  10,   2, -11,
     1,   1,   5,   9,   8,   9,   3,  -2,
     0,   0,   0,   0,   0,   0,   0,   0,
];

const PAWN_END: Weights = [
     0,   0,   0,   0,   0,   0,   0,   0,
     0,  -5,   6,  10,  13,   9,   2,   3,
    14,  10,  11,  14,  15,   3,   3,   6,
     5,   2,   2,  -2,  -2,  -2,   7,   5,
     3,  -1,  -4,  -2,  -6,  -6,  -5,  -4,
    -5,  -5,  -5,   2,   2,   1,  -3,  -2,
    -5,  -3,   5,   0,   7,   3,  -2,  -9,
     0,   0,   0,   0,   0,   0,   0,   0,
];

const KNIGHT_BASE: HalfWeights = [
    -77, -32, -22, -10,
    -26, -10,   2,  14,
     -3,   8,  22,  21,
    -13,   5,  17,  20,
    -13,   3,  15,  19,
    -23,  -7,   2,   5,
    -30, -16, -10,  -6,
    -67, -35, -28, -28,
];

const KNIGHT_END: HalfWeights = [
    -35, -31, -20,  -6,
    -24, -18, -18,   4,
    -18, -15,  -6,   6,
    -16,  -6,   3,  13,
    -12,  -1,   5,  35,
    -14,  -9,  -3,  10,
    -24, -19,  -6,   3,
    -34, -22, -17,  -7,
];

const KING_BASE: HalfWeights = [
     23,  35,  18,   0,
     35,  48,  26,  13,
     49,  58,  32,  12,
     61,  72,  42,  28,
     66,  76,  55,  39,
     78, 103,  67,  48,
    110, 120,  93,  71,
    110, 130, 110,  80,
];

const KING_END: HalfWeights = [
      5,  24,  30,  32,
     19,  48,  46,  52,
     36,  68,  74,  76,
     39,  67,  80,  80,
     41,  62,  68,  68,
     35,  52,  67,  70,
     21,  40,  53,  54,
      0,  18,  34,  30,
];

const BISHOP_BASE: HalfWeights = [
    -19,   0,  -5,  -9,
     -7,  -5,   2,   0,
     -6,   2,   0,   4,
     -5,  11,   8,  12,
     -2,   4,   9,  15,
     -3,   8,  -2,  -7,
     -6,   3,   7,   2,
    -21,  -2,  -3,  -9,
];

const BISHOP_END: HalfWeights = [
    -16, -14, -12,  -8,
    -10,  -7,  -1,   0,
    -10,   2,   1,   2,
     -6,   0,  -5,   6,
     -7,  -2,   0,   6,
     -5,   0,  -1,   4,
    -13,  -5,  -6,   0,
    -20, -11, -13,  -4,
];

const ROOK_BASE: HalfWeights = [
     -7,  -8,   0,   4,
      0,   5,   6,   7,
     -9,  -1,   2,   5,
    -11,  -6,  -2,   1,
     -5,  -2,  -1,  -2,
    -10,  -4,   0,   1,
     -8,  -5,  -3,   2,
    -31,  -8,  -5,  -2,
];

const ROOK_END: HalfWeights = [
      7,   0,   7,   5,
      1,   2,   7,  -2,
      2,   0,  -2,   4,
     -1,   3,   3,  -2,
     -2,   0,  -3,   3,
      2,  -3,  -1,  -2,
     -4,  -3,   0,  -1,
     -3,  -5,  -4,  -3,
];

const QUEEN_BASE: HalfWeights = [
     -1,  -1,   0,  -1,
     -2,   2,   4,   3,
     -1,   4,   2,   3,
      0,   5,   4,   2,
      1,   2,   3,   3,
     -1,   2,   5,   2,
     -1,   2,   3,   4,
      1,  -2,  -2,   1,
];

const QUEEN_END: HalfWeights = [
    -25, -17, -14, -12,
    -17,  -9,  -8,  -3,
    -13,  -6,  -4,   0,
    -10,  -2,   3,   7,
     -8,  -1,   4,   8,
    -13,  -6,  -3,   1,
    -18, -10,  -7,  -1,
    -23, -19, -15,  -9,
];

macro_rules! mirror_weights {
    ($x:ident) => {[
        $x[0 + 0], $x[0 + 1], $x[0 + 2], $x[0 + 3], $x[0 + 3], $x[0 + 2], $x[0 + 1], $x[0 + 0],
        $x[4 + 0], $x[4 + 1], $x[4 + 2], $x[4 + 3], $x[4 + 3], $x[4 + 2], $x[4 + 1], $x[4 + 0],
        $x[8 + 0], $x[8 + 1], $x[8 + 2], $x[8 + 3], $x[8 + 3], $x[8 + 2], $x[8 + 1], $x[8 + 0],
        $x[12 + 0], $x[12 + 1], $x[12 + 2], $x[12 + 3], $x[12 + 3], $x[12 + 2], $x[12 + 1], $x[12 + 0],
        $x[16 + 0], $x[16 + 1], $x[16 + 2], $x[16 + 3], $x[16 + 3], $x[16 + 2], $x[16 + 1], $x[16 + 0],
        $x[20 + 0], $x[20 + 1], $x[20 + 2], $x[20 + 3], $x[20 + 3], $x[20 + 2], $x[20 + 1], $x[20 + 0],
        $x[24 + 0], $x[24 + 1], $x[24 + 2], $x[24 + 3], $x[24 + 3], $x[24 + 2], $x[24 + 1], $x[24 + 0],
        $x[28 + 0], $x[28 + 1], $x[28 + 2], $x[28 + 3], $x[28 + 3], $x[28 + 2], $x[28 + 1], $x[28 + 0],
    ]};
}

macro_rules! rev_weights {
    ($x:expr) => { [
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
    ($x:expr) => { [ rev_weights!($x), $x ] };
}

macro_rules! construct_weights {
    ($base:ident, $pawn:ident, $knight:ident, $bishop:ident, $rook:ident, $queen:ident, $king:ident) => {
        WeightSet {
            base: $base,
            pawn: double_weights!($pawn),
            knight: double_weights!(mirror_weights!($knight)),
            bishop: double_weights!(mirror_weights!($bishop)),
            rook: double_weights!(mirror_weights!($rook)),
            queen: double_weights!(mirror_weights!($queen)),
            king: double_weights!(mirror_weights!($king)),
        }
    };
}

pub const MID_GAME: WeightSet =
    construct_weights!(BASE_SCORES, PAWN_BASE, KNIGHT_BASE, BISHOP_BASE, ROOK_BASE, QUEEN_BASE, KING_BASE);
pub const END_GAME: WeightSet =
    construct_weights!(END_SCORES, PAWN_END, KNIGHT_END, BISHOP_END, ROOK_END, QUEEN_END, KING_END);