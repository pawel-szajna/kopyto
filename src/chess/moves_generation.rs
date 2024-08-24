use super::board::{Board, Side, BLACK, WHITE};
use super::masks;
use super::moves::{Move, Piece, Promotion};
use super::util;

#[allow(private_bounds)]
pub trait MoveGenerator: MoveGeneratorImpl {
    fn generate_moves(&self) -> Vec<Move> {
        self.generate_moves_impl()
    }

    fn generate_moves_for(&self, file: usize, rank: usize) -> Vec<Move> {
        self.generate_moves_for_impl(util::coords_to_mask(file, rank))
    }
}

impl MoveGenerator for Board {}

trait MoveGeneratorImpl {
    fn generate_moves_impl(&self) -> Vec<Move>;
    fn generate_moves_for_impl(&self, mask: u64) -> Vec<Move>;

    fn generate_mask_moves(&self, side: Side, mask: u64, targets: &[u64; 64]) -> Vec<Move>;

    fn generate_pawn_moves(&self, side: Side, mask: u64) -> Vec<Move>;
    fn generate_knight_moves(&self, side: Side, mask: u64) -> Vec<Move>;
    fn generate_king_moves(&self, side: Side, mask: u64) -> Vec<Move>;
}

impl MoveGeneratorImpl for Board {
    fn generate_moves_impl(&self) -> Vec<Move> {
        Vec::<Move>::new()
    }

    fn generate_moves_for_impl(&self, mask: u64) -> Vec<Move> {
        match self.check_square(mask) {
            None => vec![],
            Some((side, piece)) => match piece {
                Piece::Pawn => self.generate_pawn_moves(side, mask),
                Piece::Knight => self.generate_knight_moves(side, mask),
                Piece::King => self.generate_king_moves(side, mask),
                _ => vec![],
            },
        }
    }

    fn generate_pawn_moves(&self, side: Side, mask: u64) -> Vec<Move> {
        let mut moves = vec![];

        let basic_direction = if side == WHITE { mask << 8 } else { mask >> 8 };
        let blockade = self.has_piece(basic_direction);

        if !blockade {
            moves.push(Move::from_mask(mask, basic_direction));
        }

        let piece_to_left = if side == WHITE { mask << 7 } else { mask >> 9 };
        let piece_to_right = if side == WHITE { mask << 9 } else { mask >> 7 };
        let opponent = if side == WHITE { BLACK } else { WHITE };

        if masks::FILES[0] & mask == 0
            && (self.has_side_piece(opponent, piece_to_left) || self.en_passant(piece_to_left))
        {
            moves.push(Move::from_mask(mask, piece_to_left));
        }
        if masks::FILES[7] & mask == 0
            && (self.has_side_piece(opponent, piece_to_right) || self.en_passant(piece_to_right))
        {
            moves.push(Move::from_mask(mask, piece_to_right));
        }

        if masks::RANKS_RELATIVE[6][side] & mask != 0 {
            moves = moves
                .into_iter()
                .flat_map(|m| {
                    std::iter::repeat(m)
                        .take(4)
                        .zip([
                            Promotion::Queen,
                            Promotion::Rook,
                            Promotion::Bishop,
                            Promotion::Knight,
                        ])
                        .map(|(m, p)| {
                            let mut m = m.clone();
                            m.set_promotion(p);
                            m
                        })
                })
                .collect();
        }

        let double_move_target = if side == WHITE {
            mask << 16
        } else {
            mask >> 16
        };

        if !blockade
            && masks::RANKS_RELATIVE[1][side] & mask != 0
            && !self.has_piece(double_move_target)
        {
            moves.push(Move::from_mask(mask, double_move_target));
        }

        moves
    }

    fn generate_mask_moves(&self, side: Side, mask: u64, targets: &[u64; 64]) -> Vec<Move> {
        let mut moves = vec![];
        let mut moves_mask = targets[mask.trailing_zeros() as usize] & !self.occupied[side];

        while moves_mask != 0 {
            let extracted = 1u64 << moves_mask.trailing_zeros();
            moves.push(Move::from_mask(mask, extracted));
            moves_mask ^= extracted;
        }

        moves
    }

    fn generate_knight_moves(&self, side: Side, mask: u64) -> Vec<Move> {
        self.generate_mask_moves(side, mask, &masks::KNIGHT_TARGETS)
    }

    fn generate_king_moves(&self, side: Side, mask: u64) -> Vec<Move> {
        let mut moves = self.generate_mask_moves(side, mask, &masks::KING_TARGETS);

        if self.can_castle_kingside(side) {
            moves.push(Move::from_mask(mask, masks::CASTLE_KINGSIDE[side]));
        }
        if self.can_castle_queenside(side) {
            moves.push(Move::from_mask(mask, masks::CASTLE_QUEENSIDE[side]));
        }

        moves
    }
}

macro_rules! a_move {
    ($from:expr,$to:expr) => {
        Move::from_str($from, $to)
    };
    ($from:expr,$to:expr,$prom:expr) => {
        Move::from_str_prom($from, $to, $prom)
    };
}

mod tests {
    use super::*;

    fn piece_move_generation_test(fen: &str, file: usize, rank: usize, mut expected: Vec<Move>) {
        let board = Board::from_fen(fen);

        let mut generated = board.generate_moves_for(file, rank);

        generated.sort_unstable();
        expected.sort_unstable();

        assert_eq!(generated, expected);
    }

    mod pawn {
        use super::*;

        #[test]
        fn basic_moves() {
            piece_move_generation_test(
                "4k3/2p5/8/8/8/3P4/2P5/7K w - - 0 1",
                2,
                1,
                vec![a_move!("c2", "c3"), a_move!("c2", "c4")],
            );
            piece_move_generation_test(
                "4k3/2p5/8/8/8/3P4/2P5/7K w - - 0 1",
                3,
                2,
                vec![a_move!("d3", "d4")],
            );
            piece_move_generation_test(
                "4k3/2p5/8/8/8/3P4/2P5/7K b - - 0 1",
                2,
                6,
                vec![a_move!("c7", "c6"), a_move!("c7", "c5")],
            );
        }

        #[test]
        fn blockade() {
            piece_move_generation_test(
                "rnbqk1nr/ppp1pppp/8/4b3/4P3/3p3R/PPPP1PPP/RNBQKBN1 w Qkq - 0 1",
                4,
                3,
                vec![],
            );
            piece_move_generation_test(
                "rnbqk1nr/ppp1pppp/8/4b3/4P3/3p3R/PPPP1PPP/RNBQKBN1 w Qkq - 0 1",
                3,
                1,
                vec![],
            );
            piece_move_generation_test(
                "rnbqk1nr/ppp1pppp/8/4b3/4P3/3p3R/PPPP1PPP/RNBQKBN1 w Qkq - 0 1",
                7,
                1,
                vec![],
            );
        }

        #[test]
        fn captures() {
            piece_move_generation_test(
                "rnbqkbnr/pppp1ppp/8/4p3/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 1",
                3,
                3,
                vec![a_move!("d4", "d5"), a_move!("d4", "e5")],
            );
            piece_move_generation_test(
                "rnbqk2r/pppppppp/8/3n1b2/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1",
                4,
                3,
                vec![
                    a_move!("e4", "d5"),
                    a_move!("e4", "e5"),
                    a_move!("e4", "f5"),
                ],
            );
            piece_move_generation_test(
                "rnbqk2r/pppppppp/8/3nRb2/4P3/8/PPPP1PPP/RNBQKBN1 w Qkq - 0 1",
                4,
                3,
                vec![a_move!("e4", "d5"), a_move!("e4", "f5")],
            );
            piece_move_generation_test(
                "rnbqk2r/pppp1ppp/4p3/3nRP2/4P3/8/PPPP2PP/RNBQKBN1 b Qkq - 0 1",
                4,
                5,
                vec![a_move!("e6", "f5")],
            );
            piece_move_generation_test(
                "rnbqk2r/pppp2pp/7R/Pp5p/P6n/8/PPPP2PP/RNBQKBN1 w Qkq - 0 1",
                0,
                3,
                vec![a_move!("a4", "b5")],
            );
            piece_move_generation_test(
                "rnbqk2r/pppp2pp/7R/Pp5p/P6n/8/PPPP2PP/RNBQKBN1 w Qkq - 0 1",
                0,
                6,
                vec![a_move!("a7", "a6")],
            );
        }

        #[test]
        fn promotions() {
            piece_move_generation_test(
                "8/3P4/8/8/8/8/8/8 w - - 0 1",
                3,
                6,
                vec![
                    a_move!("d7", "d8", Promotion::Knight),
                    a_move!("d7", "d8", Promotion::Bishop),
                    a_move!("d7", "d8", Promotion::Rook),
                    a_move!("d7", "d8", Promotion::Queen),
                ],
            );
            piece_move_generation_test(
                "rnbqkbnr/pPpppppp/8/8/8/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1",
                1,
                6,
                vec![
                    a_move!("b7", "a8", Promotion::Queen),
                    a_move!("b7", "a8", Promotion::Rook),
                    a_move!("b7", "a8", Promotion::Bishop),
                    a_move!("b7", "a8", Promotion::Knight),
                    a_move!("b7", "c8", Promotion::Queen),
                    a_move!("b7", "c8", Promotion::Rook),
                    a_move!("b7", "c8", Promotion::Bishop),
                    a_move!("b7", "c8", Promotion::Knight),
                ],
            );
        }

        #[test]
        fn en_passant() {
            piece_move_generation_test(
                "rnbqkbnr/pp1pp1pp/8/1Pp2p2/8/8/P1PPPPPP/RNBQKBNR w KQkq c6 0 3",
                1,
                4,
                vec![a_move!("b5", "b6"), a_move!("b5", "c6")],
            );
            piece_move_generation_test(
                "rnbqkbnr/p2pp1pp/1p6/1Pp2p2/8/3P4/P1P1PPPP/RNBQKBNR w KQkq c6 0 4",
                1,
                4,
                vec![a_move!("b5", "c6")],
            );
            piece_move_generation_test(
                "rnbqkb1r/p2pp1pp/1pP2n2/8/5pP1/3P1N2/P1P1PP1P/RNBQKB1R b KQkq g3 0 6",
                5,
                3,
                vec![a_move!("f4", "g3")],
            );
        }
    }

    mod knight {
        use super::*;

        #[test]
        fn basic_moves() {
            piece_move_generation_test(
                "7k/8/3n4/8/2N5/8/8/7K w - - 0 1",
                2,
                3,
                vec![
                    a_move!("c4", "b6"),
                    a_move!("c4", "d6"),
                    a_move!("c4", "a5"),
                    a_move!("c4", "e5"),
                    a_move!("c4", "a3"),
                    a_move!("c4", "e3"),
                    a_move!("c4", "b2"),
                    a_move!("c4", "d2"),
                ],
            );
            piece_move_generation_test(
                "7k/8/1Q1n4/8/2N5/P3P3/1P1P4/7K w - - 0 1",
                2,
                3,
                vec![
                    a_move!("c4", "d6"),
                    a_move!("c4", "a5"),
                    a_move!("c4", "e5"),
                ],
            );
            piece_move_generation_test(
                "7k/8/1Q1n4/8/2N5/P3P3/1P1P4/7K b - - 0 1",
                3,
                5,
                vec![
                    a_move!("d6", "c8"),
                    a_move!("d6", "e8"),
                    a_move!("d6", "b7"),
                    a_move!("d6", "f7"),
                    a_move!("d6", "b5"),
                    a_move!("d6", "f5"),
                    a_move!("d6", "c4"),
                    a_move!("d6", "e4"),
                ],
            );
        }

        #[test]
        fn edge_moves() {
            piece_move_generation_test(
                "n6k/8/8/8/4P3/P2P4/1P6/7K b - - 2 4",
                0,
                7,
                vec![a_move!("a8", "c7"), a_move!("a8", "b6")],
            );
            piece_move_generation_test(
                "7k/1n6/8/8/4P3/P2P4/1P6/7K b - - 10 8",
                1,
                6,
                vec![
                    a_move!("b7", "d8"),
                    a_move!("b7", "d6"),
                    a_move!("b7", "c5"),
                    a_move!("b7", "a5"),
                ],
            );
            piece_move_generation_test(
                "rnbqkbnr/1ppppppp/p7/8/8/7N/PPPPPPPP/RNBQKB1R w KQkq - 0 2",
                7,
                2,
                vec![
                    a_move!("h3", "g5"),
                    a_move!("h3", "f4"),
                    a_move!("h3", "g1"),
                ],
            );
        }
    }

    mod king {
        use super::*;

        #[test]
        fn basic_moves() {
            piece_move_generation_test(
                "8/p6k/8/8/8/8/1K6/8 w - - 0 1",
                1,
                1,
                vec![
                    a_move!("b2", "a1"),
                    a_move!("b2", "a2"),
                    a_move!("b2", "a3"),
                    a_move!("b2", "b1"),
                    a_move!("b2", "b3"),
                    a_move!("b2", "c1"),
                    a_move!("b2", "c2"),
                    a_move!("b2", "c3"),
                ],
            );
            piece_move_generation_test(
                "8/p6k/8/8/8/8/1K6/8 b - - 0 1",
                7,
                6,
                vec![
                    a_move!("h7", "g8"),
                    a_move!("h7", "h8"),
                    a_move!("h7", "g7"),
                    a_move!("h7", "g6"),
                    a_move!("h7", "h6"),
                ],
            );
            piece_move_generation_test(
                "7k/p7/8/8/8/8/8/K7 w - - 4 3",
                0,
                0,
                vec![
                    a_move!("a1", "a2"),
                    a_move!("a1", "b2"),
                    a_move!("a1", "b1"),
                ],
            );
        }

        #[test]
        fn castling() {
            piece_move_generation_test(
                "rn2kb1r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1",
                4,
                0,
                vec![
                    a_move!("e1", "d1"),
                    a_move!("e1", "f1"),
                    a_move!("e1", "g1"),
                    a_move!("e1", "c1"),
                ],
            );
            piece_move_generation_test(
                "rn2kb1r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w Qkq - 0 1",
                4,
                0,
                vec![
                    a_move!("e1", "d1"),
                    a_move!("e1", "f1"),
                    a_move!("e1", "c1"),
                ],
            );
            piece_move_generation_test(
                "rn2kb1r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w Kkq - 0 1",
                4,
                0,
                vec![
                    a_move!("e1", "d1"),
                    a_move!("e1", "f1"),
                    a_move!("e1", "g1"),
                ],
            );
            piece_move_generation_test(
                "rn2kb1r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w q - 0 1",
                4,
                0,
                vec![a_move!("e1", "d1"), a_move!("e1", "f1")],
            );
            piece_move_generation_test(
                "rn2kb1r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R b kq - 0 1",
                4,
                7,
                vec![a_move!("e8", "d8")],
            )
        }
    }
}
