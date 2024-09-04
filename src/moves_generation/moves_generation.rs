use crate::board::Board;
use crate::masks;
use crate::types::{Bitboard, Move, Promotion, Side, Square};
use std::time::SystemTime;

pub trait MoveGenerator: pimpl::MoveGenerator {
    fn generate_moves(&mut self, captures_only: bool) -> Vec<Move>;

    fn generate_side_moves(&mut self, side: Side, captures_only: bool) -> Vec<Move> {
        self.generate_moves_impl(side, captures_only)
    }

    fn generate_attacks(&mut self, side: Side) -> Bitboard {
        self.attack_mask(side)
    }

    #[cfg(test)]
    fn generate_moves_for(&mut self, file: usize, rank: usize) -> Vec<Move>;
    #[cfg(test)]
    fn generate_side_moves_for(&mut self, side: Side, file: usize, rank: usize) -> Vec<Move> {
        self.generate_moves_for_impl(side, Bitboard::from_coords(file, rank))
    }
}

impl MoveGenerator for Board {
    fn generate_moves(&mut self, captures_only: bool) -> Vec<Move> {
        self.generate_side_moves(self.side_to_move(), captures_only)
    }

    #[cfg(test)]
    fn generate_moves_for(&mut self, file: usize, rank: usize) -> Vec<Move> {
        self.generate_side_moves_for(self.side_to_move(), file, rank)
    }
}

pub fn perft(board: &mut Board, depth: usize) -> u64 {
    let start = SystemTime::now();
    let nodes = perft_impl(board, depth, true);
    let time_taken = start.elapsed().unwrap();
    let nps = if time_taken.as_nanos() > 0 {
        1000000000 * nodes as u128 / time_taken.as_nanos()
    } else {
        0
    };
    println!(
        "depth {} nodes {} time {} nps {}",
        depth,
        nodes,
        time_taken.as_millis(),
        if nps != 0 {
            format!("{}", nps)
        } else {
            String::from("?")
        }
    );
    nodes
}

fn perft_impl(board: &mut Board, depth: usize, init: bool) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut nodes = 0;
    let moves = board.generate_moves(false);
    for m in moves {
        board.make_move(m.clone());
        let res = perft_impl(board, depth - 1, false);
        if init {
            println!("{:?}: {}", m, res);
        }
        nodes += res;
        board.unmake_move();
    }

    nodes
}

mod attacks {
    use super::*;
    use crate::board::magics;

    pub fn pawn(side: Side, idx: Square) -> Bitboard {
        masks::PAWN_TARGETS[side][idx]
    }

    pub fn knight(idx: Square) -> Bitboard {
        masks::KNIGHT_TARGETS[idx]
    }

    pub fn bishop(idx: Square, occupied: Bitboard) -> Bitboard {
        Bitboard::from_u64(magics::BISHOP_MAGICS.get(idx as usize, occupied))
    }

    pub fn rook(idx: Square, occupied: Bitboard) -> Bitboard {
        Bitboard::from_u64(magics::ROOK_MAGICS.get(idx as usize, occupied))
    }

    pub fn king(idx: Square) -> Bitboard {
        masks::KING_TARGETS[idx]
    }
}

mod pimpl {
    use super::*;
    use crate::types::Piece;

    fn generate_piece(moves: &mut Vec<Move>, mask: Bitboard, generator: impl Fn(Square) -> Bitboard)
    {
        for src_idx in mask {
            for tgt_idx in generator(src_idx) {
                moves.push(Move::from_idx(src_idx, tgt_idx));
            }
        }
    }

    fn push_pawns(side: Side, pawns: Bitboard) -> Bitboard {
        match side {
            Side::White => pawns << 8,
            Side::Black => pawns >> 8,
        }
    }

    pub trait MoveGenerator {
        fn generate_moves_impl(&mut self, side: Side, captures_only: bool) -> Vec<Move>;

        #[cfg(test)]
        fn generate_moves_for_impl(&mut self, side: Side, mask: Bitboard) -> Vec<Move>;

        fn check_mask(&self, side: Side, king: Bitboard) -> (u64, Bitboard);
        fn attack_mask(&self, side: Side) -> Bitboard;
        fn pin_mask(&self, side: Side, king_idx: Square, attacks: Bitboard) -> Bitboard;
        fn parallel_pin_mask(&self, side: Side, king_idx: Square) -> Bitboard;
        fn diagonal_pin_mask(&self, side: Side, king_idx: Square) -> Bitboard;

        fn generate_pawns(
            &self,
            side: Side,
            moves: &mut Vec<Move>,
            parallel_pin_mask: Bitboard,
            diagonal_pin_mask: Bitboard,
            check_mask: Bitboard,
            captures_only: bool,
        );
        fn generate_knight(&self, knight_idx: Square) -> Bitboard;
        fn generate_king(&self, king_idx: Square, side: Side, legal_mask: Bitboard) -> Bitboard;
        fn generate_rook(&self, rook_idx: Square, parallel_pin_mask: Bitboard) -> Bitboard;
        fn generate_bishop(&self, bishop_idx: Square, diagonal_pin_mask: Bitboard) -> Bitboard;
        fn generate_queen(&self, queen_idx: Square, parallel_pin_mask: Bitboard, diagonal_pin_mask: Bitboard) -> Bitboard;
    }

    impl MoveGenerator for Board {
        fn generate_moves_impl(&mut self, side: Side, captures_only: bool) -> Vec<Move> {
            let opponent = !side;

            let mut moves = Vec::with_capacity(64);

            let (check_count, check_mask) = self.check_mask(side, self.kings[side]);

            let parallel_pin_mask = self.parallel_pin_mask(side, self.kings[side].peek());
            let diagonal_pin_mask = self.diagonal_pin_mask(side, self.kings[side].peek());

            let mut legal_targets = match captures_only {
                true => self.occupied[opponent],
                false => !self.occupied[side],
            };

            generate_piece(&mut moves, self.kings[side], |idx| self.generate_king(idx, side, legal_targets));

            if check_count > 1 {
                return moves;
            }

            legal_targets &= check_mask;

            self.generate_pawns(side, &mut moves, parallel_pin_mask, diagonal_pin_mask, check_mask, captures_only);

            generate_piece(&mut moves, self.knights[side] & !(parallel_pin_mask | diagonal_pin_mask), |idx| {
                self.generate_knight(idx) & legal_targets
            });
            generate_piece(&mut moves, self.bishops[side] & !parallel_pin_mask, |idx| {
                self.generate_bishop(idx, diagonal_pin_mask) & legal_targets
            });
            generate_piece(&mut moves, self.rooks[side] & !diagonal_pin_mask, |idx| {
                self.generate_rook(idx, parallel_pin_mask) & legal_targets
            });
            generate_piece(&mut moves, self.queens[side] & !(parallel_pin_mask & diagonal_pin_mask), |idx| {
                self.generate_queen(idx, parallel_pin_mask, diagonal_pin_mask) & legal_targets
            });

            moves
        }

        #[cfg(test)]
        fn generate_moves_for_impl(&mut self, side: Side, mask: Bitboard) -> Vec<Move> {
            let mut moves = self.generate_moves_impl(side, false);
            moves.retain(|m| m.get_from() == mask.peek());
            moves
        }

        fn check_mask(&self, side: Side, king: Bitboard) -> (u64, Bitboard) {
            let opponent = !side;
            assert!(king.not_empty(), "no king on board");
            let king_idx = king.peek();

            let mut checks = 0;
            let mut check_mask = Bitboard::EMPTY;

            let pawns = self.pawns[opponent] & attacks::pawn(side, king_idx);
            if pawns.not_empty() {
                checks += 1;
                check_mask |= pawns;
            }

            let knights = self.knights[opponent] & attacks::knight(king_idx);
            if knights.not_empty() {
                checks += 1;
                check_mask |= knights;
            }

            let bishops = (self.bishops[opponent] | self.queens[opponent]) & attacks::bishop(king_idx, self.any_piece);
            if bishops.not_empty() {
                checks += 1;
                let attacker_idx = bishops.peek();
                check_mask |= masks::BETWEEN[king_idx][attacker_idx] | Bitboard::from(attacker_idx);
            }

            let rooks = (self.rooks[opponent] | self.queens[opponent]) & attacks::rook(king_idx, self.any_piece);
            if rooks.not_empty() {
                checks += rooks.count() as u64;
                let attacker_idx = rooks.peek();
                check_mask |= masks::BETWEEN[king_idx][attacker_idx] | Bitboard::from(attacker_idx);
            }

            if check_mask.empty() {
                check_mask = !check_mask;
            }

            (checks, check_mask)
        }

        fn attack_mask(&self, side: Side) -> Bitboard {
            let opponent = !side;

            let king_idx = self.kings[opponent].peek();
            let king_attacks = attacks::king(king_idx);

            if (king_attacks & !self.occupied[opponent]).empty() {
                // king cannot move
                return Bitboard::EMPTY;
            }

            let mut mask = Bitboard::EMPTY;
            let occupied = self.any_piece & !self.kings[opponent];

            let pawns = self.pawns[side];
            for pawn_idx in pawns {
                mask |= attacks::pawn(side, pawn_idx);
            }

            let knights = self.knights[side];
            for knight_idx in knights {
                mask |= attacks::knight(knight_idx);
            }

            let bishops = self.bishops[side] | self.queens[side];
            for bishop_idx in bishops {
                mask |= attacks::bishop(bishop_idx, occupied);
            }

            let rooks = self.rooks[side] | self.queens[side];
            for rook_idx in rooks {
                mask |= attacks::rook(rook_idx, occupied);
            }

            mask |= attacks::king(self.kings[side].peek());

            mask
        }

        fn pin_mask(&self, side: Side, king_idx: Square, attacks: Bitboard) -> Bitboard {
            let mut result = Bitboard::EMPTY;

            for pinner_idx in attacks {
                let pinner = Bitboard::from(pinner_idx);
                let ray = masks::BETWEEN[king_idx][pinner_idx] | pinner;
                if (ray & self.occupied[side]).pieces() == 1 {
                    result |= ray;
                }
            }

            result
        }

        fn parallel_pin_mask(&self, side: Side, king_idx: Square) -> Bitboard {
            let opponent = !side;

            self.pin_mask(
                side,
                king_idx,
                attacks::rook(king_idx, self.occupied[opponent]) & (self.rooks[opponent] | self.queens[opponent]))
        }

        fn diagonal_pin_mask(&self, side: Side, king_idx: Square) -> Bitboard {
            let opponent = !side;
            self.pin_mask(
                side,
                king_idx,
                attacks::bishop(king_idx, self.occupied[opponent]) & (self.bishops[opponent] | self.queens[opponent]),
            )
        }

        fn generate_pawns(
            &self,
            side: Side,
            moves: &mut Vec<Move>,
            parallel_pin_mask: Bitboard,
            diagonal_pin_mask: Bitboard,
            check_mask: Bitboard,
            captures_only: bool,
        ) {
            let opponent = !side;

            let pawns = self.pawns[side];

            let pawns_may_take = pawns & !parallel_pin_mask;
            let pawns_may_take_unpinned = pawns_may_take & !diagonal_pin_mask;
            let pawns_may_take_pinned = pawns_may_take & diagonal_pin_mask;

            let mut attacks_left = match side {
                Side::White => {
                    ((pawns_may_take_unpinned << 7) & !masks::FILES[7])
                        | ((pawns_may_take_pinned << 7) & !masks::FILES[7] & diagonal_pin_mask)
                },
                Side::Black => {
                    ((pawns_may_take_unpinned >> 7) & !masks::FILES[0])
                        | ((pawns_may_take_pinned >> 7) & !masks::FILES[0] & diagonal_pin_mask)
                },
            } & check_mask
                & self.occupied[opponent];

            let mut attacks_right = match side {
                Side::White => {
                    ((pawns_may_take_unpinned << 9) & !masks::FILES[0])
                        | ((pawns_may_take_pinned << 9) & !masks::FILES[0] & diagonal_pin_mask)
                },
                Side::Black => {
                    ((pawns_may_take_unpinned >> 9) & !masks::FILES[7])
                        | ((pawns_may_take_pinned >> 9) & !masks::FILES[7] & diagonal_pin_mask)
                },
            } & check_mask
                & self.occupied[opponent];

            let pawns_may_walk = pawns & !diagonal_pin_mask;
            let pawns_may_walk_pinned = pawns_may_walk & parallel_pin_mask;
            let pawns_may_walk_unpinned = pawns_may_walk & !parallel_pin_mask;

            let pawns_walk_unpinned = !self.any_piece & push_pawns(side, pawns_may_walk_unpinned);
            let pawns_walk_pinned = !self.any_piece & parallel_pin_mask & push_pawns(side, pawns_may_walk_pinned);

            let mut pawns_walk = (pawns_walk_unpinned | pawns_walk_pinned) & check_mask;

            let pawns_double = (pawns_walk_unpinned | pawns_walk_pinned) & masks::RANKS_RELATIVE[2][side];

            let pawns_double_walk = !self.any_piece & check_mask & push_pawns(side, pawns_double);

            if (pawns & masks::NEXT_TO_SECOND_RANK[side]).not_empty() {
                let promotion_attacks_left = attacks_left & masks::LAST_RANK[side];
                let promotion_attacks_right = attacks_right & masks::LAST_RANK[side];
                let promotion_walk = pawns_walk & masks::LAST_RANK[side];

                for tgt_idx in promotion_attacks_left {
                    let src_idx = side.choose(tgt_idx.southeast(), tgt_idx.northwest());
                    moves.push(Move::from_idx_prom(src_idx, tgt_idx, Promotion::Knight));
                    moves.push(Move::from_idx_prom(src_idx, tgt_idx, Promotion::Bishop));
                    moves.push(Move::from_idx_prom(src_idx, tgt_idx, Promotion::Rook));
                    moves.push(Move::from_idx_prom(src_idx, tgt_idx, Promotion::Queen));
                }

                for tgt_idx in promotion_attacks_right {
                    let src_idx = side.choose(tgt_idx.southwest(), tgt_idx.northeast());
                    moves.push(Move::from_idx_prom(src_idx, tgt_idx, Promotion::Knight));
                    moves.push(Move::from_idx_prom(src_idx, tgt_idx, Promotion::Bishop));
                    moves.push(Move::from_idx_prom(src_idx, tgt_idx, Promotion::Rook));
                    moves.push(Move::from_idx_prom(src_idx, tgt_idx, Promotion::Queen));
                }

                if !captures_only {
                    for tgt_idx in promotion_walk {
                        let src_idx = side.choose(tgt_idx.south(), tgt_idx.north());
                        moves.push(Move::from_idx_prom(src_idx, tgt_idx, Promotion::Knight));
                        moves.push(Move::from_idx_prom(src_idx, tgt_idx, Promotion::Bishop));
                        moves.push(Move::from_idx_prom(src_idx, tgt_idx, Promotion::Rook));
                        moves.push(Move::from_idx_prom(src_idx, tgt_idx, Promotion::Queen));
                    }
                }
            }

            pawns_walk &= !masks::LAST_RANK[side];
            attacks_left &= !masks::LAST_RANK[side];
            attacks_right &= !masks::LAST_RANK[side];

            for idx in attacks_left {
                moves.push(Move::from_idx(side.choose(idx.southeast(), idx.northwest()), idx));
            }

            for idx in attacks_right {
                moves.push(Move::from_idx(side.choose(idx.southwest(), idx.northeast()), idx));
            }

            if !captures_only {
                for idx in pawns_walk {
                    moves.push(Move::from_idx(side.choose(idx.south(), idx.north()), idx));
                }

                for idx in pawns_double_walk {
                    moves.push(Move::from_idx(side.choose(idx.south().south(), idx.north().north()), idx));
                }
            }

            if self.en_passant.empty() {
                return;
            }

            let target = self.en_passant;
            let target_idx = target.peek();
            let enemy_pawn = side.choose(target >> 8, target << 8);
            let enemy_pawn_idx = enemy_pawn.peek();

            if ((enemy_pawn | target) & check_mask).empty() {
                return;
            }

            let en_passant_attackers = attacks::pawn(opponent, target_idx) & pawns_may_take;

            let king_mask = self.kings[side] & masks::RANKS[enemy_pawn_idx.rank()];
            let rook_mask = self.rooks[opponent] | self.queens[opponent];

            for source_idx in en_passant_attackers {
                let source = Bitboard::from(source_idx);

                if (source & diagonal_pin_mask).empty() || (target & diagonal_pin_mask).not_empty() {
                    if king_mask.not_empty() && rook_mask.not_empty() {
                        let pawns_mask = enemy_pawn | source;
                        let king_idx = self.kings[side].peek();
                        if (attacks::rook(king_idx, self.any_piece & !pawns_mask) & rook_mask).not_empty() {
                            break;
                        }
                    }

                    moves.push(Move::from_idx(source_idx, target_idx));
                }
            }
        }

        fn generate_knight(&self, knight_idx: Square) -> Bitboard {
            attacks::knight(knight_idx)
        }

        fn generate_king(&self, king_idx: Square, side: Side, legal_mask: Bitboard) -> Bitboard {
            let enemy_attacks = self.attack_mask(!side);
            let mut targets = attacks::king(king_idx) & legal_mask & !enemy_attacks;

            if self.castle_kingside[side]
                && self.check_piece(side, masks::ROOK_KINGSIDE[side]) == Some(Piece::Rook)
                && (masks::KING_STARTING_POSITION[side] & enemy_attacks).empty()
                && (masks::CASTLE_KINGSIDE_BLOCKER[side] & !self.any_piece & !enemy_attacks).not_empty()
                && (masks::CASTLE_KINGSIDE[side] & legal_mask & !self.any_piece & !enemy_attacks).not_empty()
            {
                targets |= masks::CASTLE_KINGSIDE[side];
            }

            if self.castle_queenside[side]
                && !self.has_piece(masks::CASTLE_QUEENSIDE_BLOCKER_KNIGHT[side])
                && self.check_piece(side, masks::ROOK_QUEENSIDE[side]) == Some(Piece::Rook)
                && (masks::KING_STARTING_POSITION[side] & enemy_attacks).empty()
                && (masks::CASTLE_QUEENSIDE_BLOCKER_QUEEN[side] & !self.any_piece & !enemy_attacks).not_empty()
                && (masks::CASTLE_QUEENSIDE[side] & legal_mask & !self.any_piece & !enemy_attacks).not_empty()
            {
                targets |= masks::CASTLE_QUEENSIDE[side];
            }

            targets
        }

        fn generate_rook(&self, rook_idx: Square, parallel_pin_mask: Bitboard) -> Bitboard {
            match (parallel_pin_mask & Bitboard::from(rook_idx)).not_empty() {
                true => attacks::rook(rook_idx, self.any_piece) & parallel_pin_mask,
                false => attacks::rook(rook_idx, self.any_piece),
            }
        }

        fn generate_bishop(&self, bishop_idx: Square, diagonal_pin_mask: Bitboard) -> Bitboard {
            match (diagonal_pin_mask & Bitboard::from(bishop_idx)).not_empty() {
                true => attacks::bishop(bishop_idx, self.any_piece) & diagonal_pin_mask,
                false => attacks::bishop(bishop_idx, self.any_piece),
            }
        }

        fn generate_queen(&self, queen_idx: Square, parallel_pin_mask: Bitboard, diagonal_pin_mask: Bitboard) -> Bitboard {
            match Bitboard::from(queen_idx) {
                mask if (mask & diagonal_pin_mask).not_empty() => attacks::bishop(queen_idx, self.any_piece) & diagonal_pin_mask,
                mask if (mask & parallel_pin_mask).not_empty() => attacks::rook(queen_idx, self.any_piece) & parallel_pin_mask,
                _ => attacks::bishop(queen_idx, self.any_piece) | attacks::rook(queen_idx, self.any_piece),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::FenConsumer;

    macro_rules! a_move {
        ($from:expr,$to:expr) => {
            Move::from_str($from, $to)
        };
        ($from:expr,$to:expr,$prom:expr) => {
            Move::from_str_prom($from, $to, $prom)
        };
    }

    fn move_generation_comparison(mut generated: Vec<Move>, mut expected: Vec<Move>) {
        generated.sort_unstable();
        expected.sort_unstable();

        assert_eq!(generated, expected);
    }

    fn piece_move_generation_test(fen: &str, file: usize, rank: usize, expected: Vec<Move>) {
        println!("-- Move generation test at position {}", fen);

        let mut board = Board::from_fen(fen);
        let moves = board.generate_moves_for(file, rank);

        move_generation_comparison(moves, expected);
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

        #[test]
        fn perft_initial_6() {
            perft_initial(6, 119060324);
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

        #[test]
        fn perft_kiwipete_5() {
            perft_kiwipete(5, 193690690);
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

        #[test]
        fn perft_endgame_6() {
            perft_endgame(6, 11030083);
        }

        fn perft_pos4(depth: usize, expected: u64) {
            perft_run("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1", depth, expected);
        }

        #[test]
        fn perft_pos4_1() {
            perft_pos4(1, 6);
        }

        #[test]
        fn perft_pos4_2() {
            perft_pos4(2, 264);
        }

        #[test]
        fn perft_pos4_3() {
            perft_pos4(3, 9467);
        }

        #[test]
        fn perft_pos4_4() {
            perft_pos4(4, 422333);
        }

        #[test]
        fn perft_pos4_5() {
            perft_pos4(5, 15833292);
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

        #[test]
        fn perft_pos5_5() {
            perft_pos5(5, 89941194);
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
            piece_move_generation_test(
                "rnbqkbnr/1ppp1ppp/8/3PpP2/p7/8/PPP1P1PP/RNBQKBNR w KQkq e6 0 5",
                3,
                4,
                vec![a_move!("d5", "d6"), a_move!("d5", "e6")],
            );
            piece_move_generation_test(
                "rnbqkbnr/1ppp1ppp/8/3PpP2/p7/8/PPP1P1PP/RNBQKBNR w KQkq e6 0 5",
                5,
                4,
                vec![a_move!("f5", "e6"), a_move!("f5", "f6")],
            );
            piece_move_generation_test("8/2p5/3p4/KP5r/1R2Pp1k/8/6P1/8 b - e3 0 1", 5, 3, vec![a_move!("f4", "f3")]);
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
            );
            piece_move_generation_test(
                "r3k2r/p1ppqpb1/Bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPB1PPP/R3K2R b KQkq - 0 1",
                4,
                7,
                vec![a_move!("e8", "d8"), a_move!("e8", "f8"), a_move!("e8", "g8")],
            );
            piece_move_generation_test(
                "r3k2r/p1ppqpb1/1n2pnp1/3PN3/1p2P3/2N2Q1P/PPPBbP1P/R3K2R w KQkq - 0 2",
                4,
                0,
                vec![a_move!("e1", "e2")],
            );
            piece_move_generation_test(
                "r3k2r/p1ppqpb1/1n2pnp1/3PN3/1p2P3/2N2Q1p/PPPB1PPP/R2BKb1R w KQkq - 2 2",
                4,
                0,
                vec![a_move!("e1", "f1")],
            );
            piece_move_generation_test(
                "r1B1k2r/p1ppqpb1/1n2pnp1/3PN3/4P3/2p2Q1p/PPPB1PPP/R3K2R b KQkq - 1 2",
                4,
                7,
                vec![a_move!("e8", "d8"), a_move!("e8", "f8"), a_move!("e8", "g8")],
            );
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

    #[test]
    fn from_position() {
        let mut board = Board::from_starting_position();

        board.make_move(Move::from_uci("c2c3"));
        board.make_move(Move::from_uci("d7d6"));
        board.make_move(Move::from_uci("d1a4"));

        move_generation_comparison(
            board.generate_moves(false),
            vec![
                a_move!("b7", "b5"),
                a_move!("b8", "c6"),
                a_move!("b8", "d7"),
                a_move!("c7", "c6"),
                a_move!("c8", "d7"),
                a_move!("d8", "d7"),
            ],
        );
    }
}
