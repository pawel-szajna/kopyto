use std::time::SystemTime;
use super::board::{Board, Side, BLACK, WHITE};
use super::masks;
use super::moves::{Move, Promotion};
#[cfg(any(test, feature = "ui"))]
use super::moves::Piece;
#[cfg(any(test, feature = "ui"))]
use super::util;

pub type Moves = (Vec<Move>, u64);

pub trait MoveGenerator: pimpl::MoveGenerator {
    fn generate_moves(&mut self) -> Moves;
    fn generate_side_moves(&mut self, side: Side) -> Moves {
        self.generate_moves_impl(side)
    }

    fn prune_checks(&mut self, side: Side, moves: &mut Vec<Move>);

    #[cfg(any(test, feature = "ui"))]
    fn generate_moves_for(&mut self, file: usize, rank: usize) -> Moves;
    #[cfg(any(test, feature = "ui"))]
    fn generate_side_moves_for(&mut self, side: Side, file: usize, rank: usize) -> Moves {
        let mut result = (vec![], 0u64);
        self.generate_moves_for_impl(&mut result.0, &mut result.1, side, util::coords_to_mask(file, rank));
        result
    }
}

impl MoveGenerator for Board {
    fn generate_moves(&mut self) -> Moves {
        self.generate_side_moves(self.side_to_move())
    }

    fn prune_checks(&mut self, side: Side, moves: &mut Vec<Move>) {
        moves.retain(|m| {
            self.make_move(m.clone());
            let retain = !self.in_check(side);
            self.unmake_move();
            retain
        });
    }

    #[cfg(any(test, feature = "ui"))]
    fn generate_moves_for(&mut self, file: usize, rank: usize) -> Moves {
        self.generate_side_moves_for(self.side_to_move(), file, rank)
    }
}

pub fn perft(board: &mut Board, depth: usize) -> u64 {
    let start = SystemTime::now();
    let nodes = perft_impl(board, depth, true);
    let time_taken = start.elapsed().unwrap();
    let nps = 1000000000 * nodes as u128 / time_taken.as_nanos();
    println!("depth {} nodes {} time {} nps {}", depth, nodes, time_taken.as_millis(), nps);
    nodes
}

fn perft_impl(board: &mut Board, depth: usize, init: bool) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut nodes = 0;
    let (moves, _) = board.generate_moves();
    let side = board.side_to_move();
    for m in &moves {
        board.make_move(m.clone());
        if !board.in_check(side) {
            let res = perft_impl(board, depth - 1, false);
            if init {
                println!("{:?}: {}", m, res);
            }
            nodes += res;
        }
        board.unmake_move();
    }

    nodes
}

mod pimpl {
    use super::*;

    fn extract_mask_to_moves(from: u64, mut moves_mask: u64, moves: &mut Vec<Move>) {
        while moves_mask != 0 {
            let extracted = 1u64 << moves_mask.trailing_zeros();
            moves.push(Move::from_mask(from, extracted));
            moves_mask ^= extracted;
        }
    }

    fn collect_sliders(
        mut slider: u64,
        diff: isize,
        boundary: u64,
        friendly: u64,
        enemy: u64,
        attacked_squares: &mut u64,
    ) {
        while slider & boundary != 0 {
            if diff >= 0 {
                slider = slider.checked_shl(diff as u32).unwrap_or(0);
            } else {
                slider = slider.checked_shr(-diff as u32).unwrap_or(0);
            }
            if friendly & slider != 0 {
                break;
            }
            *attacked_squares |= slider;
            if enemy & slider != 0 {
                break;
            }
        }
    }

    pub trait MoveGenerator {
        fn generate_piece<F>(&mut self, moves: &mut Vec<Move>, attacks: &mut u64, side: Side, mask: u64, f: F)
        where
            F: Fn(&Self, &mut Vec<Move>, &mut u64, Side, u64);
        fn generate_moves_impl(&mut self, side: Side) -> Moves;

        #[cfg(any(test, feature = "ui"))]
        fn generate_moves_for_impl(&mut self, moves: &mut Vec<Move>, attacks: &mut u64, side: Side, mask: u64);

        fn generate_mask_moves(
            &self,
            moves: &mut Vec<Move>,
            attacks: &mut u64,
            side: Side,
            mask: u64,
            targets: &[u64; 64],
        );

        fn generate_pawn(&self, moves: &mut Vec<Move>, attacks: &mut u64, side: Side, mask: u64);
        fn generate_knight(&self, moves: &mut Vec<Move>, attacks: &mut u64, side: Side, mask: u64);
        fn generate_king(&mut self, moves: &mut Vec<Move>, attacks: &mut u64, side: Side, mask: u64);
        fn generate_rook(&self, moves: &mut Vec<Move>, attacks: &mut u64, side: Side, mask: u64);
        fn generate_bishop(&self, moves: &mut Vec<Move>, attacks: &mut u64, side: Side, mask: u64);
        fn generate_queen(&self, moves: &mut Vec<Move>, attacks: &mut u64, side: Side, mask: u64);
    }

    impl MoveGenerator for Board {
        fn generate_moves_impl(&mut self, side: Side) -> Moves {
            if self.moves[side].is_some() && self.attacks[side].is_some() {
                return (self.moves[side].clone().unwrap(), self.attacks[side].unwrap());
            }
            let mut moves = Vec::with_capacity(64);
            let mut attacks = 0u64;
            self.generate_piece(&mut moves, &mut attacks, side, self.pawns[side], Self::generate_pawn);
            self.generate_piece(&mut moves, &mut attacks, side, self.rooks[side], Self::generate_rook);
            self.generate_piece(&mut moves, &mut attacks, side, self.bishops[side], Self::generate_bishop);
            self.generate_piece(&mut moves, &mut attacks, side, self.queens[side], Self::generate_queen);
            self.generate_piece(&mut moves, &mut attacks, side, self.knights[side], Self::generate_knight);
            self.generate_king(&mut moves, &mut attacks, side, self.kings[side]);

            self.moves[side] = Some(moves.clone());
            self.attacks[side] = Some(attacks);
            (moves, attacks)
        }

        fn generate_piece<F>(&mut self, moves: &mut Vec<Move>, attacks: &mut u64, side: Side, mut mask: u64, f: F)
        where
            F: Fn(&Self, &mut Vec<Move>, &mut u64, Side, u64),
        {
            while mask != 0 {
                let extracted = 1u64 << mask.trailing_zeros();
                f(self, moves, attacks, side, extracted);
                mask ^= extracted;
            }
        }

        #[cfg(any(test, feature = "ui"))]
        fn generate_moves_for_impl(&mut self, moves: &mut Vec<Move>, attacks: &mut u64, side: Side, mask: u64) {
            match self.check_piece(side, mask) {
                None => (),
                Some(piece) => match piece {
                    Piece::Pawn => self.generate_pawn(moves, attacks, side, mask),
                    Piece::Knight => self.generate_knight(moves, attacks, side, mask),
                    Piece::King => self.generate_king(moves, attacks, side, mask),
                    Piece::Rook => self.generate_rook(moves, attacks, side, mask),
                    Piece::Bishop => self.generate_bishop(moves, attacks, side, mask),
                    Piece::Queen => self.generate_queen(moves, attacks, side, mask),
                },
            }
        }

        fn generate_mask_moves(
            &self,
            moves: &mut Vec<Move>,
            attacks: &mut u64,
            side: Side,
            mask: u64,
            targets: &[u64; 64],
        ) {
            let moves_mask = targets[mask.trailing_zeros() as usize] & !self.occupied[side];
            extract_mask_to_moves(mask, moves_mask, moves);
            *attacks |= moves_mask;
        }

        fn generate_pawn(&self, moves: &mut Vec<Move>, attacks: &mut u64, side: Side, mask: u64) {
            let basic_direction = if side == WHITE { mask << 8 } else { mask >> 8 };
            let blockade = self.has_piece(basic_direction);
            let mut pawn_moves = [Move::new(), Move::new(), Move::new()];
            let mut pawn_moves_used = 0usize;

            if !blockade {
                pawn_moves[pawn_moves_used] = Move::from_mask(mask, basic_direction);
                pawn_moves_used += 1;
            }

            let piece_to_left = if side == WHITE { mask << 7 } else { mask >> 9 };
            let piece_to_right = if side == WHITE { mask << 9 } else { mask >> 7 };
            let opponent = if side == WHITE { BLACK } else { WHITE };

            let possible_pawn_attacks = self.occupied[opponent] | self.en_passant;

            if masks::FILES[0] & mask == 0 {
                *attacks |= piece_to_left;
                if possible_pawn_attacks & piece_to_left != 0 {
                    pawn_moves[pawn_moves_used] = Move::from_mask(mask, piece_to_left);
                    pawn_moves_used += 1;
                }
            }

            if masks::FILES[7] & mask == 0 {
                *attacks |= piece_to_right;
                if possible_pawn_attacks & piece_to_right != 0 {
                    pawn_moves[pawn_moves_used] = Move::from_mask(mask, piece_to_right);
                    pawn_moves_used += 1;
                }
            }

            if masks::RANKS_RELATIVE[6][side] & mask != 0 {
                pawn_moves[0..pawn_moves_used]
                    .into_iter()
                    .flat_map(|m| {
                        std::iter::repeat(m)
                            .take(4)
                            .zip([Promotion::Queen, Promotion::Rook, Promotion::Bishop, Promotion::Knight])
                            .map(|(m, p)| {
                                let mut m = m.clone();
                                m.set_promotion(p);
                                m
                            })
                    })
                    .for_each(|m| {
                        moves.push(m);
                    });
            } else {
                pawn_moves[0..pawn_moves_used].into_iter().for_each(|m| {
                    moves.push(m.clone());
                });
            }

            let double_move_target = if side == WHITE { mask << 16 } else { mask >> 16 };

            if !blockade && masks::RANKS_RELATIVE[1][side] & mask != 0 && !self.has_piece(double_move_target) {
                moves.push(Move::from_mask(mask, double_move_target));
            }
        }

        fn generate_knight(&self, moves: &mut Vec<Move>, attacks: &mut u64, side: Side, mask: u64) {
            self.generate_mask_moves(moves, attacks, side, mask, &masks::KNIGHT_TARGETS);
        }

        fn generate_king(&mut self, moves: &mut Vec<Move>, attacks: &mut u64, side: Side, mask: u64) {
            self.generate_mask_moves(moves, attacks, side, mask, &masks::KING_TARGETS);

            if self.can_castle_kingside(side) {
                moves.push(Move::from_mask(mask, masks::CASTLE_KINGSIDE[side]));
            }
            if self.can_castle_queenside(side) {
                moves.push(Move::from_mask(mask, masks::CASTLE_QUEENSIDE[side]));
            }
        }

        fn generate_rook(&self, moves: &mut Vec<Move>, attacks: &mut u64, side: Side, mask: u64) {
            let opponent = if side == WHITE { BLACK } else { WHITE };

            let mut attacked = 0u64;

            let idx = mask.trailing_zeros() as usize;
            let file = idx % 8;
            let rank = idx / 8;

            collect_sliders(
                mask,
                -1,
                masks::RANKS[rank] & !masks::FILES[0],
                self.occupied[side],
                self.occupied[opponent],
                &mut attacked,
            );
            collect_sliders(
                mask,
                1,
                masks::RANKS[rank] & !masks::FILES[7],
                self.occupied[side],
                self.occupied[opponent],
                &mut attacked,
            );
            collect_sliders(
                mask,
                8,
                masks::FILES[file] & !masks::RANKS[7],
                self.occupied[side],
                self.occupied[opponent],
                &mut attacked,
            );
            collect_sliders(
                mask,
                -8,
                masks::FILES[file] & !masks::RANKS[0],
                self.occupied[side],
                self.occupied[opponent],
                &mut attacked,
            );

            *attacks |= attacked;
            extract_mask_to_moves(mask, attacked, moves);
        }

        fn generate_bishop(&self, moves: &mut Vec<Move>, attacks: &mut u64, side: Side, mask: u64) {
            let opponent = if side == WHITE { BLACK } else { WHITE };

            let mut attacked = 0u64;

            collect_sliders(
                mask,
                -7,
                !(masks::RANKS[0] | masks::FILES[7]),
                self.occupied[side],
                self.occupied[opponent],
                &mut attacked,
            );
            collect_sliders(
                mask,
                -9,
                !(masks::RANKS[0] | masks::FILES[0]),
                self.occupied[side],
                self.occupied[opponent],
                &mut attacked,
            );
            collect_sliders(
                mask,
                7,
                !(masks::RANKS[7] | masks::FILES[0]),
                self.occupied[side],
                self.occupied[opponent],
                &mut attacked,
            );
            collect_sliders(
                mask,
                9,
                !(masks::RANKS[7] | masks::FILES[7]),
                self.occupied[side],
                self.occupied[opponent],
                &mut attacked,
            );

            *attacks |= attacked;
            extract_mask_to_moves(mask, attacked, moves);
        }

        fn generate_queen(&self, moves: &mut Vec<Move>, attacks: &mut u64, side: Side, mask: u64) {
            self.generate_rook(moves, attacks, side, mask);
            self.generate_bishop(moves, attacks, side, mask);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! a_move {
        ($from:expr,$to:expr) => {
            Move::from_str($from, $to)
        };
        ($from:expr,$to:expr,$prom:expr) => {
            Move::from_str_prom($from, $to, $prom)
        };
    }

    fn piece_move_generation_test(fen: &str, file: usize, rank: usize, mut expected: Vec<Move>) {
        let mut board = Board::from_fen(fen);

        let mut moves = board.generate_moves_for(file, rank);
        board.prune_checks(board.side_to_move(), &mut moves.0);

        let mut generated = moves.0;

        generated.sort_unstable();
        expected.sort_unstable();

        assert_eq!(generated, expected);
    }

    mod perft {
        use super::*;

        fn perft_run(fen: &str, depth: usize, expected: u64) {
            let mut board = Board::from_fen(fen);
            assert_eq!(perft(&mut board, depth), expected);
        }

        fn perft_initial(depth: usize, expected: u64) {
            perft_run("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", depth, expected);
        }

        #[test]
        fn perft_initial_0() {
            perft_initial(0, 1);
        }

        #[test]
        fn perft_initial_1() {
            perft_initial(1, 20);
        }

        #[test]
        fn perft_initial_2() {
            perft_initial(2, 400);
        }

        #[test]
        fn perft_initial_3() {
            perft_initial(3, 8902);
        }

        #[test]
        fn perft_initial_4() {
            perft_initial(4, 197281);
        }

        #[test]
        fn perft_initial_5() {
            perft_initial(5, 4865609);
        }

        fn perft_kiwipete(depth: usize, expected: u64) {
            perft_run("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 0", depth, expected);
        }

        #[test]
        fn perft_kiwipete_1() {
            perft_kiwipete(1, 48);
        }

        #[test]
        fn perft_kiwipete_2() {
            perft_kiwipete(2, 2039);
        }

        #[test]
        fn perft_kiwipete_3() {
            perft_kiwipete(3, 97862);
        }

        #[test]
        fn perft_kiwipete_4() {
            perft_kiwipete(4, 4085603);
        }

        fn perft_endgame(depth: usize, expected: u64) {
            perft_run("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 0", depth, expected);
        }

        #[test]
        fn perft_endgame_1() {
            perft_endgame(1, 14);
        }

        #[test]
        fn perft_endgame_2() {
            perft_endgame(2, 191);
        }

        #[test]
        fn perft_endgame_3() {
            perft_endgame(3, 2812);
        }

        #[test]
        fn perft_endgame_4() {
            perft_endgame(4, 43238);
        }

        #[test]
        fn perft_endgame_5() {
            perft_endgame(5, 674624);
        }

        fn perft_pos5(depth: usize, expected: u64) {
            perft_run("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", depth, expected);
        }

        #[test]
        fn perft_pos5_1() {
            perft_pos5(1, 44);
        }

        #[test]
        fn perft_pos5_2() {
            perft_pos5(2, 1486);
        }

        #[test]
        fn perft_pos5_3() {
            perft_pos5(3, 62379);
        }

        #[test]
        fn perft_pos5_4() {
            perft_pos5(4, 2103487);
        }
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
            piece_move_generation_test("4k3/2p5/8/8/8/3P4/2P5/7K w - - 0 1", 3, 2, vec![a_move!("d3", "d4")]);
            piece_move_generation_test(
                "4k3/2p5/8/8/8/3P4/2P5/7K b - - 0 1",
                2,
                6,
                vec![a_move!("c7", "c6"), a_move!("c7", "c5")],
            );
        }

        #[test]
        fn blockade() {
            piece_move_generation_test("rnbqk1nr/ppp1pppp/8/4b3/4P3/3p3R/PPPP1PPP/RNBQKBN1 w Qkq - 0 1", 4, 3, vec![]);
            piece_move_generation_test("rnbqk1nr/ppp1pppp/8/4b3/4P3/3p3R/PPPP1PPP/RNBQKBN1 w Qkq - 0 1", 3, 1, vec![]);
            piece_move_generation_test("rnbqk1nr/ppp1pppp/8/4b3/4P3/3p3R/PPPP1PPP/RNBQKBN1 w Qkq - 0 1", 7, 1, vec![]);
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
                vec![a_move!("e4", "d5"), a_move!("e4", "e5"), a_move!("e4", "f5")],
            );
            piece_move_generation_test(
                "rnbqk2r/pppppppp/8/3nRb2/4P3/8/PPPP1PPP/RNBQKBN1 w Qkq - 0 1",
                4,
                3,
                vec![a_move!("e4", "d5"), a_move!("e4", "f5")],
            );
            piece_move_generation_test("rnbqk2r/pppp1ppp/4p3/3nRP2/4P3/8/PPPP2PP/RNBQKBN1 b Qkq - 0 1", 4, 5, vec![]);
            piece_move_generation_test(
                "rnbqk2r/pppp2pp/7R/Pp5p/P6n/8/PPPP2PP/RNBQKBN1 w Qkq - 0 1",
                0,
                3,
                vec![a_move!("a4", "b5")],
            );
            piece_move_generation_test(
                "rnbqk2r/pppp2pp/7R/Pp5p/P6n/8/PPPP2PP/RNBQKBN1 b Qkq - 0 1",
                0,
                6,
                vec![a_move!("a7", "a6")],
            );
        }

        #[test]
        fn promotions() {
            piece_move_generation_test(
                "8/3P4/8/8/8/8/8/k6K w - - 0 1",
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
                vec![a_move!("c4", "d6"), a_move!("c4", "a5"), a_move!("c4", "e5")],
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
                vec![a_move!("h3", "g5"), a_move!("h3", "f4"), a_move!("h3", "g1")],
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
                vec![a_move!("a1", "a2"), a_move!("a1", "b2"), a_move!("a1", "b1")],
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
                vec![a_move!("e1", "d1"), a_move!("e1", "f1"), a_move!("e1", "c1")],
            );
            piece_move_generation_test(
                "rn2kb1r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w Kkq - 0 1",
                4,
                0,
                vec![a_move!("e1", "d1"), a_move!("e1", "f1"), a_move!("e1", "g1")],
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

        #[test]
        fn check_evasion() {
            piece_move_generation_test(
                "rn2k1nr/ppp3pp/8/2P2pK1/P6P/4b1p1/4P3/1N3q2 w kq - 3 21",
                6,
                4,
                vec![a_move!("g5", "h5")],
            );
        }
    }
}
