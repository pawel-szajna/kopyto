use scanner_rust::ScannerAscii;
use crate::board::Board;
use crate::search::Score;
use crate::types::{Bitboard, Piece, Side};

type W = i16;
const ZERO: W = 0;

const L1_DIM: usize = 768;
const L2_DIM: usize = 64;
const L3_DIM: usize = 32;

const CHESS_BOARD_SIZE: usize = 64;
const CHESS_PIECE_TYPES: usize = 6;

#[derive(Clone)]
pub struct NN {
    l1_weights: [[W; L1_DIM]; L2_DIM],
    l1_bias: [W; L2_DIM],

    l2_weights: [[W; L2_DIM]; L3_DIM],
    l2_bias: [W; L3_DIM],

    l3_weights: [[W; L3_DIM]; 1],
    l3_bias: W,
}

fn load_dim1<const SIZE: usize>(source: &mut ScannerAscii<std::fs::File>, target: &mut [W; SIZE]) {
    assert_eq!(source.next_i8().unwrap().unwrap(), 1);
    assert_eq!(source.next_usize().unwrap().unwrap(), SIZE);
    for i in 0..SIZE {
        target[i] = source.next_i16().unwrap().unwrap();
    }
}

fn load_dim2<const SIZE_OUTER: usize, const SIZE_INNER: usize>(
    source: &mut ScannerAscii<std::fs::File>,
    target: &mut [[W; SIZE_INNER]; SIZE_OUTER]
) {
    assert_eq!(source.next_i8().unwrap().unwrap(), 2);
    assert_eq!(source.next_usize().unwrap().unwrap(), SIZE_OUTER);
    for i in 0..SIZE_OUTER {
        assert_eq!(source.next_usize().unwrap().unwrap(), SIZE_INNER);
        for j in 0..SIZE_INNER {
            target[i][j] = source.next_i16().unwrap().unwrap();
        }
    }
}

impl NN {
    pub fn load(source: &str) -> Self {
        let file = std::fs::File::open(source);
        if let Err(cause) = file {
            panic!("Failed to open file: {}", cause);
        }

        let mut nn = Self::empty();
        let mut scanner = ScannerAscii::new(file.unwrap());

        assert_eq!(scanner.next_usize().unwrap().unwrap(), 6); // layer * (weights + bias)

        load_dim2(&mut scanner, &mut nn.l1_weights);
        load_dim1(&mut scanner, &mut nn.l1_bias);
        load_dim2(&mut scanner, &mut nn.l2_weights);
        load_dim1(&mut scanner, &mut nn.l2_bias);
        load_dim2(&mut scanner, &mut nn.l3_weights);

        assert_eq!(scanner.next_usize().unwrap().unwrap(), 1); // L3 data dimension
        assert_eq!(scanner.next_usize().unwrap().unwrap(), 1); // L3 bias size
        nn.l3_bias = scanner.next_i16().unwrap().unwrap();

        nn
    }

    fn empty() -> Self {
        Self {
            l1_weights: [[ZERO; L1_DIM]; L2_DIM],
            l1_bias: [ZERO; L2_DIM],

            l2_weights: [[ZERO; L2_DIM]; L3_DIM],
            l2_bias: [ZERO; L3_DIM],

            l3_weights: [[ZERO; L3_DIM]; 1],
            l3_bias: ZERO,
        }
    }

    pub fn eval(&self, board: &Board) -> Score {
        let mut accumulator = [ZERO; L1_DIM];

        for side in [Side::White, Side::Black] {
            let color_mod = CHESS_BOARD_SIZE * CHESS_PIECE_TYPES * match side {
                Side::White => 1,
                Side::Black => 0,
            };

            for idx in board.occupied[side] {
                let piece_type = board.check_piece(side, Bitboard::from(idx));

                let piece_mod = CHESS_BOARD_SIZE * match unsafe { piece_type.unwrap_unchecked() } {
                    Piece::Pawn => 0,
                    Piece::Knight => 1,
                    Piece::Bishop => 2,
                    Piece::Rook => 3,
                    Piece::Queen => 4,
                    Piece::King => 5,
                };

                let accumulator_idx = color_mod + piece_mod + idx as usize;

                accumulator[accumulator_idx] = 1;
            }
        }

        let mut l2 = self.l1_bias;
        for i in 0..L2_DIM {
            for j in 0..L1_DIM {
                l2[i] += accumulator[j] * self.l1_weights[i][j];
            }
        }
        for i in 0..L2_DIM {
            l2[i] = l2[i].clamp(0, 127);
        }

        let mut l3 = self.l2_bias;
        for i in 0..L3_DIM {
            for j in 0..L2_DIM {
                l3[i] += ((l2[j] as i32 * self.l2_weights[i][j] as i32) >> 7) as i16;
            }
        }
        for i in 0..L3_DIM {
            l3[i] = l3[i].clamp(0, 127);
        }

        let out = self.l3_bias as i32 + l3
            .iter()
            .zip(&self.l3_weights[0])
            .map(|(l3value, weight)| *l3value as i32 * *weight as i32)
            .sum::<i32>() / 127;

        out as Score
    }
}
