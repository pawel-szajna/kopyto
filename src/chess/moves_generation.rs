use super::board::{Board, Side, BLACK, WHITE};
use super::masks;
use super::moves::{Move, Promotion};
#[cfg(any(test, feature = "ui"))]
use super::util;
use std::time::SystemTime;

pub trait MoveGenerator: pimpl::MoveGenerator {
    fn generate_moves(&mut self, captures_only: bool) -> Vec<Move>;

    fn generate_side_moves(&mut self, side: Side, captures_only: bool) -> Vec<Move> {
        self.generate_moves_impl(side, captures_only)
    }

    fn generate_attacks(&mut self, side: Side) -> u64 {
        self.attack_mask(side)
    }

    #[cfg(any(test, feature = "ui"))]
    fn generate_moves_for(&mut self, file: usize, rank: usize) -> Vec<Move>;
    #[cfg(any(test, feature = "ui"))]
    fn generate_side_moves_for(&mut self, side: Side, file: usize, rank: usize) -> Vec<Move> {
        self.generate_moves_for_impl(side, util::coords_to_mask(file, rank))
    }
}

impl MoveGenerator for Board {
    fn generate_moves(&mut self, captures_only: bool) -> Vec<Move> {
        self.generate_side_moves(self.side_to_move(), captures_only)
    }

    #[cfg(any(test, feature = "ui"))]
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
    use crate::chess::magics::Magics;
    use super::*;

    pub fn pawn(side: Side, idx: usize) -> u64 {
        masks::PAWN_TARGETS[side][idx]
    }

    pub fn knight(idx: usize) -> u64 {
        masks::KNIGHT_TARGETS[idx]
    }

    pub fn bishop(idx: usize, occupied: u64, magics: &Magics) -> u64 {
        magics.get(idx, occupied)
    }

    pub fn rook(idx: usize, occupied: u64, magics: &Magics) -> u64 {
        magics.get(idx, occupied)
    }

    pub fn king(idx: usize) -> u64 {
        masks::KING_TARGETS[idx]
    }
}

mod pimpl {
    use super::*;
    use crate::chess::moves::Piece;

    fn generate_piece<F>(moves: &mut Vec<Move>, mut mask: u64, generator: F)
    where
        F: Fn(usize) -> u64,
    {
        while mask != 0 {
            let src_idx = mask.trailing_zeros() as usize;
            let mut moves_mask = generator(src_idx);
            while moves_mask != 0 {
                let tgt_idx = moves_mask.trailing_zeros() as usize;
                moves.push(Move::from_idx(src_idx, tgt_idx));
                moves_mask &= moves_mask - 1;
            }
            mask &= mask - 1;
        }
    }

    pub trait MoveGenerator {
        fn generate_moves_impl(&mut self, side: Side, captures_only: bool) -> Vec<Move>;

        #[cfg(any(test, feature = "ui"))]
        fn generate_moves_for_impl(&mut self, side: Side, mask: u64) -> Vec<Move>;

        fn check_mask(&self, side: Side, king: u64) -> (u64, u64);
        fn attack_mask(&self, side: Side) -> u64;
        fn pin_mask(&self, side: Side, king_idx: usize, attacks: u64) -> u64;
        fn parallel_pin_mask(&self, side: Side, king: u64) -> u64;
        fn diagonal_pin_mask(&self, side: Side, king: u64) -> u64;

        fn generate_pawns(
            &self,
            side: Side,
            moves: &mut Vec<Move>,
            parallel_pin_mask: u64,
            diagonal_pin_mask: u64,
            check_mask: u64,
            captures_only: bool,
        );
        fn generate_knight(&self, knight_idx: usize) -> u64;
        fn generate_king(&self, king_idx: usize, side: Side, legal_mask: u64) -> u64;
        fn generate_rook(&self, rook_idx: usize, parallel_pin_mask: u64) -> u64;
        fn generate_bishop(&self, bishop_idx: usize, diagonal_pin_mask: u64) -> u64;
        fn generate_queen(&self, queen_idx: usize, parallel_pin_mask: u64, diagonal_pin_mask: u64) -> u64;
    }

    impl MoveGenerator for Board {
        fn generate_moves_impl(&mut self, side: Side, captures_only: bool) -> Vec<Move> {
            let opponent = if side == WHITE { BLACK } else { WHITE };

            let mut moves = Vec::with_capacity(64);

            let (check_count, check_mask) = self.check_mask(side, self.kings[side]);

            let parallel_pin_mask = self.parallel_pin_mask(side, self.kings[side]);
            let diagonal_pin_mask = self.diagonal_pin_mask(side, self.kings[side]);

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

        #[cfg(any(test, feature = "ui"))]
        fn generate_moves_for_impl(&mut self, side: Side, mask: u64) -> Vec<Move> {
            let mut moves = self.generate_moves_impl(side, false);
            moves.retain(|m| m.get_from() == mask.trailing_zeros() as u16);
            moves
        }

        fn check_mask(&self, side: Side, king: u64) -> (u64, u64) {
            let opponent = if side == WHITE { BLACK } else { WHITE };
            assert_ne!(king, 0, "no king on board");
            let king_idx = king.trailing_zeros() as usize;

            let mut checks = 0;
            let mut check_mask = 0;

            let pawns = self.pawns[opponent] & attacks::pawn(side, king_idx);
            if pawns != 0 {
                checks += 1;
                check_mask |= pawns;
            }

            let knights = self.knights[opponent] & attacks::knight(king_idx);
            if knights != 0 {
                checks += 1;
                check_mask |= knights;
            }

            let bishops = (self.bishops[opponent] | self.queens[opponent]) & attacks::bishop(king_idx, self.any_piece, &self.bishop_magics);
            if bishops != 0 {
                checks += 1;
                let attacker_idx = bishops.trailing_zeros() as usize;
                check_mask |= masks::BETWEEN[king_idx][attacker_idx] | (1u64 << attacker_idx);
            }

            let rooks = (self.rooks[opponent] | self.queens[opponent]) & attacks::rook(king_idx, self.any_piece, &self.rook_magics);
            if rooks != 0 {
                checks += rooks.count_ones() as u64;
                let attacker_idx = rooks.trailing_zeros() as usize;
                check_mask |= masks::BETWEEN[king_idx][attacker_idx] | (1u64 << attacker_idx);
            }

            if check_mask == 0 {
                check_mask = !check_mask;
            }

            (checks, check_mask)
        }

        fn attack_mask(&self, side: Side) -> u64 {
            let opponent = if side == WHITE { BLACK } else { WHITE };

            let king_idx = self.kings[opponent].trailing_zeros() as usize;
            let king_attacks = attacks::king(king_idx);

            if king_attacks & !self.occupied[opponent] == 0 {
                // king cannot move
                return 0;
            }

            let mut mask = 0;
            let occupied = self.any_piece & !self.kings[opponent];

            let mut pawns = self.pawns[side];
            while pawns != 0 {
                let pawn_idx = pawns.trailing_zeros() as usize;
                mask |= attacks::pawn(side, pawn_idx);
                pawns &= pawns - 1;
            }

            let mut knights = self.knights[side];
            while knights != 0 {
                let knight_idx = knights.trailing_zeros() as usize;
                mask |= attacks::knight(knight_idx);
                knights &= knights - 1;
            }

            let mut bishops = self.bishops[side] | self.queens[side];
            while bishops != 0 {
                let bishop_idx = bishops.trailing_zeros() as usize;
                mask |= attacks::bishop(bishop_idx, occupied, &self.bishop_magics);
                bishops &= bishops - 1;
            }

            let mut rooks = self.rooks[side] | self.queens[side];
            while rooks != 0 {
                let rook_idx = rooks.trailing_zeros() as usize;
                mask |= attacks::rook(rook_idx, occupied, &self.rook_magics);
                rooks &= rooks - 1;
            }

            mask |= attacks::king(self.kings[side].trailing_zeros() as usize);

            mask
        }

        fn pin_mask(&self, side: Side, king_idx: usize, mut attacks: u64) -> u64 {
            let mut result = 0;

            while attacks != 0 {
                let pinner_idx = attacks.trailing_zeros() as usize;
                let pinner = 1u64 << pinner_idx;
                let ray = masks::BETWEEN[king_idx][pinner_idx] | pinner;
                if (ray & self.occupied[side]).count_ones() == 1 {
                    result |= ray;
                }
                attacks ^= pinner;
            }

            result
        }

        fn parallel_pin_mask(&self, side: Side, king: u64) -> u64 {
            let king_idx = king.trailing_zeros() as usize;
            let opponent = if side == WHITE { BLACK } else { WHITE };
            self.pin_mask(
                side,
                king_idx,
                attacks::rook(king_idx, self.occupied[opponent], &self.rook_magics) & (self.rooks[opponent] | self.queens[opponent]),
            )
        }

        fn diagonal_pin_mask(&self, side: Side, king: u64) -> u64 {
            let king_idx = king.trailing_zeros() as usize;
            let opponent = if side == WHITE { BLACK } else { WHITE };
            self.pin_mask(
                side,
                king_idx,
                attacks::bishop(king_idx, self.occupied[opponent], &self.bishop_magics) & (self.bishops[opponent] | self.queens[opponent]),
            )
        }

        fn generate_pawns(
            &self,
            side: Side,
            moves: &mut Vec<Move>,
            parallel_pin_mask: u64,
            diagonal_pin_mask: u64,
            check_mask: u64,
            captures_only: bool,
        ) {
            let opponent = if side == WHITE { BLACK } else { WHITE };

            let pawns = self.pawns[side];

            let pawns_may_take = pawns & !parallel_pin_mask;
            let pawns_may_take_unpinned = pawns_may_take & !diagonal_pin_mask;
            let pawns_may_take_pinned = pawns_may_take & diagonal_pin_mask;

            let mut attacks_left = match side {
                WHITE => {
                    ((pawns_may_take_unpinned << 7) & !masks::FILES[7])
                        | ((pawns_may_take_pinned << 7) & !masks::FILES[7] & diagonal_pin_mask)
                }
                _ => {
                    ((pawns_may_take_unpinned >> 7) & !masks::FILES[0])
                        | ((pawns_may_take_pinned >> 7) & !masks::FILES[0] & diagonal_pin_mask)
                }
            } & check_mask
                & self.occupied[opponent];

            let mut attacks_right = match side {
                WHITE => {
                    ((pawns_may_take_unpinned << 9) & !masks::FILES[0])
                        | ((pawns_may_take_pinned << 9) & !masks::FILES[0] & diagonal_pin_mask)
                }
                _ => {
                    ((pawns_may_take_unpinned >> 9) & !masks::FILES[7])
                        | ((pawns_may_take_pinned >> 9) & !masks::FILES[7] & diagonal_pin_mask)
                }
            } & check_mask
                & self.occupied[opponent];

            let pawns_may_walk = pawns & !diagonal_pin_mask;
            let pawns_may_walk_pinned = pawns_may_walk & parallel_pin_mask;
            let pawns_may_walk_unpinned = pawns_may_walk & !parallel_pin_mask;

            let pawns_walk_unpinned = if side == WHITE {
                pawns_may_walk_unpinned << 8
            } else {
                pawns_may_walk_unpinned >> 8
            } & !self.any_piece;
            let pawns_walk_pinned = if side == WHITE {
                pawns_may_walk_pinned << 8
            } else {
                pawns_may_walk_pinned >> 8
            } & !self.any_piece
                & parallel_pin_mask;

            let mut pawns_walk = (pawns_walk_unpinned | pawns_walk_pinned) & check_mask;

            let pawns_double = (pawns_walk_unpinned | pawns_walk_pinned) & masks::RANKS_RELATIVE[2][side];

            let mut pawns_double_walk = if side == WHITE {
                pawns_double << 8
            } else {
                pawns_double >> 8
            } & !self.any_piece
                & check_mask;

            if pawns & masks::NEXT_TO_SECOND_RANK[side] != 0 {
                let mut promotion_attacks_left = attacks_left & masks::LAST_RANK[side];
                let mut promotion_attacks_right = attacks_right & masks::LAST_RANK[side];
                let mut promotion_walk = pawns_walk & masks::LAST_RANK[side];

                while promotion_attacks_left != 0 {
                    let tgt_idx = promotion_attacks_left.trailing_zeros() as usize;
                    let src_idx = if side == WHITE { tgt_idx - 7 } else { tgt_idx + 7 };
                    moves.push(Move::from_idx_prom(src_idx, tgt_idx, Promotion::Knight));
                    moves.push(Move::from_idx_prom(src_idx, tgt_idx, Promotion::Bishop));
                    moves.push(Move::from_idx_prom(src_idx, tgt_idx, Promotion::Rook));
                    moves.push(Move::from_idx_prom(src_idx, tgt_idx, Promotion::Queen));
                    promotion_attacks_left &= promotion_attacks_left - 1;
                }

                while promotion_attacks_right != 0 {
                    let tgt_idx = promotion_attacks_right.trailing_zeros() as usize;
                    let src_idx = if side == WHITE { tgt_idx - 9 } else { tgt_idx + 9 };
                    moves.push(Move::from_idx_prom(src_idx, tgt_idx, Promotion::Knight));
                    moves.push(Move::from_idx_prom(src_idx, tgt_idx, Promotion::Bishop));
                    moves.push(Move::from_idx_prom(src_idx, tgt_idx, Promotion::Rook));
                    moves.push(Move::from_idx_prom(src_idx, tgt_idx, Promotion::Queen));
                    promotion_attacks_right &= promotion_attacks_right - 1;
                }

                if !captures_only {
                    while promotion_walk != 0 {
                        let tgt_idx = promotion_walk.trailing_zeros() as usize;
                        let src_idx = if side == WHITE { tgt_idx - 8 } else { tgt_idx + 8 };
                        moves.push(Move::from_idx_prom(src_idx, tgt_idx, Promotion::Knight));
                        moves.push(Move::from_idx_prom(src_idx, tgt_idx, Promotion::Bishop));
                        moves.push(Move::from_idx_prom(src_idx, tgt_idx, Promotion::Rook));
                        moves.push(Move::from_idx_prom(src_idx, tgt_idx, Promotion::Queen));
                        promotion_walk &= promotion_walk - 1;
                    }
                }
            }

            pawns_walk &= !masks::LAST_RANK[side];
            attacks_left &= !masks::LAST_RANK[side];
            attacks_right &= !masks::LAST_RANK[side];

            while attacks_left != 0 {
                let idx = attacks_left.trailing_zeros() as usize;
                moves.push(Move::from_idx(if side == WHITE { idx - 7 } else { idx + 7 }, idx));
                attacks_left &= attacks_left - 1;
            }

            while attacks_right != 0 {
                let idx = attacks_right.trailing_zeros() as usize;
                moves.push(Move::from_idx(if side == WHITE { idx - 9 } else { idx + 9 }, idx));
                attacks_right &= attacks_right - 1;
            }

            if !captures_only {
                while pawns_walk != 0 {
                    let idx = pawns_walk.trailing_zeros() as usize;
                    moves.push(Move::from_idx(if side == WHITE { idx - 8 } else { idx + 8 }, idx));
                    pawns_walk &= pawns_walk - 1;
                }

                while pawns_double_walk != 0 {
                    let idx = pawns_double_walk.trailing_zeros() as usize;
                    moves.push(Move::from_idx(if side == WHITE { idx - 16 } else { idx + 16 }, idx));
                    pawns_double_walk &= pawns_double_walk - 1;
                }
            }

            if self.en_passant == 0 {
                return;
            }

            let target = self.en_passant;
            let target_idx = target.trailing_zeros() as usize;
            let enemy_pawn = if side == WHITE {
                self.en_passant >> 8
            } else {
                self.en_passant << 8
            };
            let enemy_pawn_idx = enemy_pawn.trailing_zeros() as usize;

            if (enemy_pawn | target) & check_mask == 0 {
                return;
            }

            let mut en_passant_attackers = attacks::pawn(opponent, target_idx) & pawns_may_take;

            let king_mask = self.kings[side] & masks::RANKS[enemy_pawn_idx / 8];
            let rook_mask = self.rooks[opponent] | self.queens[opponent];

            while en_passant_attackers != 0 {
                let source_idx = en_passant_attackers.trailing_zeros() as usize;
                let source = 1u64 << source_idx;
                en_passant_attackers ^= source;

                if (source & diagonal_pin_mask) == 0 || (target & diagonal_pin_mask) != 0 {
                    if king_mask != 0 && rook_mask != 0 {
                        let pawns_mask = enemy_pawn | source;
                        let king_idx = self.kings[side].trailing_zeros() as usize;
                        if attacks::rook(king_idx, self.any_piece & !pawns_mask, &self.rook_magics) & rook_mask != 0 {
                            break;
                        }
                    }

                    moves.push(Move::from_idx(source_idx, target_idx));
                }
            }
        }

        fn generate_knight(&self, knight_idx: usize) -> u64 {
            attacks::knight(knight_idx)
        }

        fn generate_king(&self, king_idx: usize, side: Side, legal_mask: u64) -> u64 {
            let enemy_attacks = self.attack_mask(if side == WHITE { BLACK } else { WHITE });
            let mut targets = attacks::king(king_idx) & legal_mask & !enemy_attacks;

            if self.castle_kingside[side]
                && self.check_piece(side, masks::ROOK_KINGSIDE[side]) == Some(Piece::Rook)
                && masks::KING_STARTING_POSITION[side] & enemy_attacks == 0
                && masks::CASTLE_KINGSIDE_BLOCKER[side] & !self.any_piece & !enemy_attacks != 0
                && masks::CASTLE_KINGSIDE[side] & legal_mask & !self.any_piece & !enemy_attacks != 0
            {
                targets |= masks::CASTLE_KINGSIDE[side];
            }

            if self.castle_queenside[side]
                && !self.has_piece(masks::CASTLE_QUEENSIDE_BLOCKER_KNIGHT[side])
                && self.check_piece(side, masks::ROOK_QUEENSIDE[side]) == Some(Piece::Rook)
                && masks::KING_STARTING_POSITION[side] & enemy_attacks == 0
                && masks::CASTLE_QUEENSIDE_BLOCKER_QUEEN[side] & !self.any_piece & !enemy_attacks != 0
                && masks::CASTLE_QUEENSIDE[side] & legal_mask & !self.any_piece & !enemy_attacks != 0
            {
                targets |= masks::CASTLE_QUEENSIDE[side];
            }

            targets
        }

        fn generate_rook(&self, rook_idx: usize, parallel_pin_mask: u64) -> u64 {
            match parallel_pin_mask & (1u64 << rook_idx) != 0 {
                true => attacks::rook(rook_idx, self.any_piece, &self.rook_magics) & parallel_pin_mask,
                false => attacks::rook(rook_idx, self.any_piece, &self.rook_magics),
            }
        }

        fn generate_bishop(&self, bishop_idx: usize, diagonal_pin_mask: u64) -> u64 {
            match diagonal_pin_mask & (1u64 << bishop_idx) != 0 {
                true => attacks::bishop(bishop_idx, self.any_piece, &self.bishop_magics) & diagonal_pin_mask,
                false => attacks::bishop(bishop_idx, self.any_piece, &self.bishop_magics),
            }
        }

        fn generate_queen(&self, queen_idx: usize, parallel_pin_mask: u64, diagonal_pin_mask: u64) -> u64 {
            match 1 << queen_idx {
                mask if mask & diagonal_pin_mask != 0 => attacks::bishop(queen_idx, self.any_piece, &self.bishop_magics) & diagonal_pin_mask,
                mask if mask & parallel_pin_mask != 0 => attacks::rook(queen_idx, self.any_piece, &self.rook_magics) & parallel_pin_mask,
                _ => attacks::bishop(queen_idx, self.any_piece, &self.bishop_magics) | attacks::rook(queen_idx, self.any_piece, &self.rook_magics),
            }
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

    fn move_generation_comparison(mut generated: Vec<Move>, mut expected: Vec<Move>) {
        generated.sort_unstable();
        expected.sort_unstable();

        assert_eq!(generated, expected);
    }

    fn piece_move_generation_test(fen: &str, file: usize, rank: usize, expected: Vec<Move>) {
        println!("Move generation test at position {}", fen);

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
