use crate::board::Board;
use crate::masks;
use crate::types::{Bitboard, Move, Piece, Promotion, Side, Square};
use crate::moves_generation::attacks;

pub type Moves = Vec<Move>;

const CAPTURES_ONLY: bool = true;
const ALL_MOVES: bool = false;

pub fn generate_all(board: &Board) -> Moves {
    generate::<ALL_MOVES>(board)
}

pub fn generate_captures(board: &Board) -> Moves {
    generate::<CAPTURES_ONLY>(board)
}

fn generate<const MODE: bool>(board: &Board) -> Moves {
    let side = board.side_to_move();
    let opponent = !side;

    let mut moves = Vec::with_capacity(64);

    let (check_count, check_mask) = check_mask(board, side, board.kings[side]);

    let parallel_pin_mask = parallel_pin_mask(board, side, board.kings[side].peek());
    let diagonal_pin_mask = diagonal_pin_mask(board, side, board.kings[side].peek());

    let mut legal_targets = match MODE {
        CAPTURES_ONLY => board.occupied[opponent],
        ALL_MOVES => !board.occupied[side],
    };

    generate_piece(&mut moves, board.kings[side], |idx| generate_king(board, idx, side, legal_targets));

    if check_count > 1 {
        return moves;
    }

    legal_targets &= check_mask;

    generate_pawns::<MODE>(board, side, &mut moves, parallel_pin_mask, diagonal_pin_mask, check_mask);

    generate_piece(&mut moves, board.knights[side] & !(parallel_pin_mask | diagonal_pin_mask), |idx| {
        generate_knight(idx) & legal_targets
    });
    generate_piece(&mut moves, board.bishops[side] & !parallel_pin_mask, |idx| {
        generate_bishop(board, idx, diagonal_pin_mask) & legal_targets
    });
    generate_piece(&mut moves, board.rooks[side] & !diagonal_pin_mask, |idx| {
        generate_rook(board, idx, parallel_pin_mask) & legal_targets
    });
    generate_piece(&mut moves, board.queens[side] & !(parallel_pin_mask & diagonal_pin_mask), |idx| {
        generate_queen(board, idx, parallel_pin_mask, diagonal_pin_mask) & legal_targets
    });

    moves
}

fn generate_piece(moves: &mut Moves, mask: Bitboard, generator: impl Fn(Square) -> Bitboard)
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

fn check_mask(board: &Board, side: Side, king: Bitboard) -> (u64, Bitboard) {
    let opponent = !side;
    assert!(king.not_empty(), "no king on board");
    let king_idx = king.peek();

    let mut checks = 0;
    let mut check_mask = Bitboard::EMPTY;

    let pawns = board.pawns[opponent] & attacks::pawn(side, king_idx);
    if pawns.not_empty() {
        checks += 1;
        check_mask |= pawns;
    }

    let knights = board.knights[opponent] & attacks::knight(king_idx);
    if knights.not_empty() {
        checks += 1;
        check_mask |= knights;
    }

    let bishops = (board.bishops[opponent] | board.queens[opponent]) & attacks::bishop(king_idx, board.any_piece);
    if bishops.not_empty() {
        checks += 1;
        let attacker_idx = bishops.peek();
        check_mask |= masks::BETWEEN[king_idx][attacker_idx] | Bitboard::from(attacker_idx);
    }

    let rooks = (board.rooks[opponent] | board.queens[opponent]) & attacks::rook(king_idx, board.any_piece);
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

pub fn attack_mask(board: &Board, side: Side) -> Bitboard {
    let opponent = !side;

    let king_idx = board.kings[opponent].peek();
    let king_attacks = attacks::king(king_idx);

    if (king_attacks & !board.occupied[opponent]).empty() {
        // king cannot move
        return Bitboard::EMPTY;
    }

    let mut mask = Bitboard::EMPTY;
    let occupied = board.any_piece & !board.kings[opponent];

    let pawns = board.pawns[side];
    for pawn_idx in pawns {
        mask |= attacks::pawn(side, pawn_idx);
    }

    let knights = board.knights[side];
    for knight_idx in knights {
        mask |= attacks::knight(knight_idx);
    }

    let bishops = board.bishops[side] | board.queens[side];
    for bishop_idx in bishops {
        mask |= attacks::bishop(bishop_idx, occupied);
    }

    let rooks = board.rooks[side] | board.queens[side];
    for rook_idx in rooks {
        mask |= attacks::rook(rook_idx, occupied);
    }

    mask |= attacks::king(board.kings[side].peek());

    mask
}

fn pin_mask(board: &Board, side: Side, king_idx: Square, attacks: Bitboard) -> Bitboard {
    let mut result = Bitboard::EMPTY;

    for pinner_idx in attacks {
        let pinner = Bitboard::from(pinner_idx);
        let ray = masks::BETWEEN[king_idx][pinner_idx] | pinner;
        if (ray & board.occupied[side]).pieces() == 1 {
            result |= ray;
        }
    }

    result
}

fn parallel_pin_mask(board: &Board, side: Side, king_idx: Square) -> Bitboard {
    let opponent = !side;
    pin_mask(
        board,
        side,
        king_idx,
        attacks::rook(king_idx, board.occupied[opponent]) & (board.rooks[opponent] | board.queens[opponent]))
}

fn diagonal_pin_mask(board: &Board, side: Side, king_idx: Square) -> Bitboard {
    let opponent = !side;
    pin_mask(
        board,
        side,
        king_idx,
        attacks::bishop(king_idx, board.occupied[opponent]) & (board.bishops[opponent] | board.queens[opponent]),
    )
}

fn generate_pawns<const CAPTURES_ONLY: bool>(
    board: &Board, side: Side, moves: &mut Moves, parallel_pins: Bitboard, diagonal_pins: Bitboard, check_mask: Bitboard
) {
    let opponent = !side;

    let pawns = board.pawns[side];

    let pawns_may_take = pawns & !parallel_pins;
    let pawns_may_take_unpinned = pawns_may_take & !diagonal_pins;
    let pawns_may_take_pinned = pawns_may_take & diagonal_pins;

    let mut attacks_left = check_mask & board.occupied[opponent] & match side {
        Side::White => ((pawns_may_take_unpinned << 7) & !masks::FILES[7])
                     | ((pawns_may_take_pinned << 7) & !masks::FILES[7] & diagonal_pins),
        Side::Black => ((pawns_may_take_unpinned >> 7) & !masks::FILES[0])
                     | ((pawns_may_take_pinned >> 7) & !masks::FILES[0] & diagonal_pins),
    };

    let mut attacks_right = check_mask & board.occupied[opponent] & match side {
        Side::White => ((pawns_may_take_unpinned << 9) & !masks::FILES[0])
                     | ((pawns_may_take_pinned << 9) & !masks::FILES[0] & diagonal_pins),
        Side::Black => ((pawns_may_take_unpinned >> 9) & !masks::FILES[7])
                     | ((pawns_may_take_pinned >> 9) & !masks::FILES[7] & diagonal_pins),
    };

    let pawns_may_walk = pawns & !diagonal_pins;
    let pawns_may_walk_pinned = pawns_may_walk & parallel_pins;
    let pawns_may_walk_unpinned = pawns_may_walk & !parallel_pins;

    let pawns_walk_unpinned = !board.any_piece & push_pawns(side, pawns_may_walk_unpinned);
    let pawns_walk_pinned = !board.any_piece & parallel_pins & push_pawns(side, pawns_may_walk_pinned);

    let mut pawns_walk = (pawns_walk_unpinned | pawns_walk_pinned) & check_mask;

    let pawns_double = (pawns_walk_unpinned | pawns_walk_pinned) & masks::RANKS_RELATIVE[2][side];

    let pawns_double_walk = !board.any_piece & check_mask & push_pawns(side, pawns_double);

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

        if !CAPTURES_ONLY {
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

    if !CAPTURES_ONLY {
        for idx in pawns_walk {
            moves.push(Move::from_idx(side.choose(idx.south(), idx.north()), idx));
        }

        for idx in pawns_double_walk {
            moves.push(Move::from_idx(side.choose(idx.south().south(), idx.north().north()), idx));
        }
    }

    if board.en_passant.empty() {
        return;
    }

    let target = board.en_passant;
    let target_idx = target.peek();
    let enemy_pawn = side.choose(target >> 8, target << 8);
    let enemy_pawn_idx = enemy_pawn.peek();

    if ((enemy_pawn | target) & check_mask).empty() {
        return;
    }

    let en_passant_attackers = attacks::pawn(opponent, target_idx) & pawns_may_take;

    let king_mask = board.kings[side] & masks::RANKS[enemy_pawn_idx.rank()];
    let rook_mask = board.rooks[opponent] | board.queens[opponent];

    for source_idx in en_passant_attackers {
        let source = Bitboard::from(source_idx);

        if (source & diagonal_pins).empty() || (target & diagonal_pins).not_empty() {
            if king_mask.not_empty() && rook_mask.not_empty() {
                let pawns_mask = enemy_pawn | source;
                let king_idx = board.kings[side].peek();
                if (attacks::rook(king_idx, board.any_piece & !pawns_mask) & rook_mask).not_empty() {
                    break;
                }
            }

            moves.push(Move::from_idx(source_idx, target_idx));
        }
    }
}

fn generate_knight(knight_idx: Square) -> Bitboard {
    attacks::knight(knight_idx)
}

fn generate_king(board: &Board, king_idx: Square, side: Side, legal_mask: Bitboard) -> Bitboard {
    let enemy_attacks = attack_mask(board, !side);
    let mut targets = attacks::king(king_idx) & legal_mask & !enemy_attacks;

    if board.castle_kingside[side]
        && board.check_piece(side, masks::ROOK_KINGSIDE[side]) == Some(Piece::Rook)
        && (masks::KING_STARTING_POSITION[side] & enemy_attacks).empty()
        && (masks::CASTLE_KINGSIDE_BLOCKER[side] & !board.any_piece & !enemy_attacks).not_empty()
        && (masks::CASTLE_KINGSIDE[side] & legal_mask & !board.any_piece & !enemy_attacks).not_empty()
    {
        targets |= masks::CASTLE_KINGSIDE[side];
    }

    if board.castle_queenside[side]
        && !board.has_piece(masks::CASTLE_QUEENSIDE_BLOCKER_KNIGHT[side])
        && board.check_piece(side, masks::ROOK_QUEENSIDE[side]) == Some(Piece::Rook)
        && (masks::KING_STARTING_POSITION[side] & enemy_attacks).empty()
        && (masks::CASTLE_QUEENSIDE_BLOCKER_QUEEN[side] & !board.any_piece & !enemy_attacks).not_empty()
        && (masks::CASTLE_QUEENSIDE[side] & legal_mask & !board.any_piece & !enemy_attacks).not_empty()
    {
        targets |= masks::CASTLE_QUEENSIDE[side];
    }

    targets
}

fn generate_rook(board: &Board, rook_idx: Square, parallel_pins: Bitboard) -> Bitboard {
    match (parallel_pins & Bitboard::from(rook_idx)).not_empty() {
        true => attacks::rook(rook_idx, board.any_piece) & parallel_pins,
        false => attacks::rook(rook_idx, board.any_piece),
    }
}

fn generate_bishop(board: &Board, bishop_idx: Square, diagonal_pin_mask: Bitboard) -> Bitboard {
    match (diagonal_pin_mask & Bitboard::from(bishop_idx)).not_empty() {
        true => attacks::bishop(bishop_idx, board.any_piece) & diagonal_pin_mask,
        false => attacks::bishop(bishop_idx, board.any_piece),
    }
}

fn generate_queen(board: &Board, queen_idx: Square, parallel_pin_mask: Bitboard, diagonal_pin_mask: Bitboard) -> Bitboard {
    match Bitboard::from(queen_idx) {
        mask if (mask & diagonal_pin_mask).not_empty() => attacks::bishop(queen_idx, board.any_piece) & diagonal_pin_mask,
        mask if (mask & parallel_pin_mask).not_empty() => attacks::rook(queen_idx, board.any_piece) & parallel_pin_mask,
        _ => attacks::bishop(queen_idx, board.any_piece) | attacks::rook(queen_idx, board.any_piece),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::FenConsumer;

    fn generate_moves_for(board: &mut Board, mask: Bitboard) -> Moves {
        let mut moves = generate::<ALL_MOVES>(board);
        moves.retain(|m| m.get_from() == mask.peek());
        moves
    }

    macro_rules! a_move {
        ($from:expr,$to:expr) => {
            Move::from_str($from, $to)
        };
        ($from:expr,$to:expr,$prom:expr) => {
            Move::from_str_prom($from, $to, $prom)
        };
    }

    fn move_generation_comparison(mut generated: Moves, mut expected: Moves) {
        generated.sort_unstable();
        expected.sort_unstable();

        assert_eq!(generated, expected);
    }

    fn piece_move_generation_test(fen: &str, file: usize, rank: usize, expected: Moves) {
        println!("-- Move generation test at position {}", fen);

        let mut board = Board::from_fen(fen);
        let moves = generate_moves_for(&mut board, Bitboard::from_coords(file, rank));

        move_generation_comparison(moves, expected);
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
            generate::<ALL_MOVES>(&mut board),
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
