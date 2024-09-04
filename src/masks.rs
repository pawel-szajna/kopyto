use seq_macro::seq;
use crate::types::Bitboard;

pub const SINGLE_RANK: Bitboard = Bitboard::from_u64(0b11111111);
pub const SINGLE_FILE: Bitboard = Bitboard::from_u64(0x0101010101010101);

pub const RANKS: [Bitboard; 8] = [
    Bitboard::from_u64(SINGLE_RANK.bitboard << (8 * 0)),
    Bitboard::from_u64(SINGLE_RANK.bitboard << (8 * 1)),
    Bitboard::from_u64(SINGLE_RANK.bitboard << (8 * 2)),
    Bitboard::from_u64(SINGLE_RANK.bitboard << (8 * 3)),
    Bitboard::from_u64(SINGLE_RANK.bitboard << (8 * 4)),
    Bitboard::from_u64(SINGLE_RANK.bitboard << (8 * 5)),
    Bitboard::from_u64(SINGLE_RANK.bitboard << (8 * 6)),
    Bitboard::from_u64(SINGLE_RANK.bitboard << (8 * 7)),
];

pub const RANKS_RELATIVE: [[Bitboard; 2]; 8] = [
    [RANKS[0], RANKS[7]],
    [RANKS[1], RANKS[6]],
    [RANKS[2], RANKS[5]],
    [RANKS[3], RANKS[4]],
    [RANKS[4], RANKS[3]],
    [RANKS[5], RANKS[2]],
    [RANKS[6], RANKS[1]],
    [RANKS[7], RANKS[0]],
];

pub const FILES: [Bitboard; 8] = [
    Bitboard::from_u64(SINGLE_FILE.bitboard << 0),
    Bitboard::from_u64(SINGLE_FILE.bitboard << 1),
    Bitboard::from_u64(SINGLE_FILE.bitboard << 2),
    Bitboard::from_u64(SINGLE_FILE.bitboard << 3),
    Bitboard::from_u64(SINGLE_FILE.bitboard << 4),
    Bitboard::from_u64(SINGLE_FILE.bitboard << 5),
    Bitboard::from_u64(SINGLE_FILE.bitboard << 6),
    Bitboard::from_u64(SINGLE_FILE.bitboard << 7),
];

const LAST_RANK_IDX: u64 = 7 * 8;

pub const ROOK_QUEENSIDE: [Bitboard; 2] = [Bitboard::from_u64(1u64), Bitboard::from_u64(1u64 << LAST_RANK_IDX)];
pub const ROOK_KINGSIDE: [Bitboard; 2] = [Bitboard::from_u64(1u64 << 7), Bitboard::from_u64(1u64 << (LAST_RANK_IDX + 7))];
pub const ROOK_CASTLED_QUEENSIDE: [Bitboard; 2] = [Bitboard::from_u64(1u64 << 3), Bitboard::from_u64(1u64 << (LAST_RANK_IDX + 3))];
pub const ROOK_CASTLED_KINGSIDE: [Bitboard; 2] = [Bitboard::from_u64(1u64 << 5), Bitboard::from_u64(1u64 << (LAST_RANK_IDX + 5))];
pub const CASTLE_QUEENSIDE: [Bitboard; 2] = [Bitboard::from_u64(1u64 << 2), Bitboard::from_u64(1u64 << (LAST_RANK_IDX + 2))];
pub const CASTLE_QUEENSIDE_BLOCKER_QUEEN: [Bitboard; 2] = [Bitboard::from_u64(1u64 << 3), Bitboard::from_u64(1u64 << (LAST_RANK_IDX + 3))];
pub const CASTLE_QUEENSIDE_BLOCKER_KNIGHT: [Bitboard; 2] = [Bitboard::from_u64(1u64 << 1), Bitboard::from_u64(1u64 << (LAST_RANK_IDX + 1))];
pub const CASTLE_KINGSIDE: [Bitboard; 2] = [Bitboard::from_u64(1u64 << 6), Bitboard::from_u64(1u64 << (LAST_RANK_IDX + 6))];
pub const KING_STARTING_POSITION: [Bitboard; 2] = [Bitboard::from_u64(1u64 << 4), Bitboard::from_u64(1u64 << (LAST_RANK_IDX + 4))];
pub const CASTLE_KINGSIDE_BLOCKER: [Bitboard; 2] = [Bitboard::from_u64(1u64 << 5), Bitboard::from_u64(1u64 << (LAST_RANK_IDX + 5))];
pub const LAST_RANK: [Bitboard; 2] = [RANKS[7], RANKS[0]];
pub const NEXT_TO_SECOND_RANK: [Bitboard; 2] = [RANKS[6], RANKS[1]];
pub const SECOND_RANK: [Bitboard; 2] = [RANKS[1], RANKS[6]];
pub const EN_PASSANT_RANK: [Bitboard; 2] = [RANKS[3], RANKS[4]];

macro_rules! fill_mask_table {
    ($generator:ident) => {
        [
            $generator(Bitboard::from_u64(1 << 0)),
            $generator(Bitboard::from_u64(1 << 1)),
            $generator(Bitboard::from_u64(1 << 2)),
            $generator(Bitboard::from_u64(1 << 3)),
            $generator(Bitboard::from_u64(1 << 4)),
            $generator(Bitboard::from_u64(1 << 5)),
            $generator(Bitboard::from_u64(1 << 6)),
            $generator(Bitboard::from_u64(1 << 7)),
            $generator(Bitboard::from_u64(1 << 8)),
            $generator(Bitboard::from_u64(1 << 9)),
            $generator(Bitboard::from_u64(1 << 10)),
            $generator(Bitboard::from_u64(1 << 11)),
            $generator(Bitboard::from_u64(1 << 12)),
            $generator(Bitboard::from_u64(1 << 13)),
            $generator(Bitboard::from_u64(1 << 14)),
            $generator(Bitboard::from_u64(1 << 15)),
            $generator(Bitboard::from_u64(1 << 16)),
            $generator(Bitboard::from_u64(1 << 17)),
            $generator(Bitboard::from_u64(1 << 18)),
            $generator(Bitboard::from_u64(1 << 19)),
            $generator(Bitboard::from_u64(1 << 20)),
            $generator(Bitboard::from_u64(1 << 21)),
            $generator(Bitboard::from_u64(1 << 22)),
            $generator(Bitboard::from_u64(1 << 23)),
            $generator(Bitboard::from_u64(1 << 24)),
            $generator(Bitboard::from_u64(1 << 25)),
            $generator(Bitboard::from_u64(1 << 26)),
            $generator(Bitboard::from_u64(1 << 27)),
            $generator(Bitboard::from_u64(1 << 28)),
            $generator(Bitboard::from_u64(1 << 29)),
            $generator(Bitboard::from_u64(1 << 30)),
            $generator(Bitboard::from_u64(1 << 31)),
            $generator(Bitboard::from_u64(1 << 32)),
            $generator(Bitboard::from_u64(1 << 33)),
            $generator(Bitboard::from_u64(1 << 34)),
            $generator(Bitboard::from_u64(1 << 35)),
            $generator(Bitboard::from_u64(1 << 36)),
            $generator(Bitboard::from_u64(1 << 37)),
            $generator(Bitboard::from_u64(1 << 38)),
            $generator(Bitboard::from_u64(1 << 39)),
            $generator(Bitboard::from_u64(1 << 40)),
            $generator(Bitboard::from_u64(1 << 41)),
            $generator(Bitboard::from_u64(1 << 42)),
            $generator(Bitboard::from_u64(1 << 43)),
            $generator(Bitboard::from_u64(1 << 44)),
            $generator(Bitboard::from_u64(1 << 45)),
            $generator(Bitboard::from_u64(1 << 46)),
            $generator(Bitboard::from_u64(1 << 47)),
            $generator(Bitboard::from_u64(1 << 48)),
            $generator(Bitboard::from_u64(1 << 49)),
            $generator(Bitboard::from_u64(1 << 50)),
            $generator(Bitboard::from_u64(1 << 51)),
            $generator(Bitboard::from_u64(1 << 52)),
            $generator(Bitboard::from_u64(1 << 53)),
            $generator(Bitboard::from_u64(1 << 54)),
            $generator(Bitboard::from_u64(1 << 55)),
            $generator(Bitboard::from_u64(1 << 56)),
            $generator(Bitboard::from_u64(1 << 57)),
            $generator(Bitboard::from_u64(1 << 58)),
            $generator(Bitboard::from_u64(1 << 59)),
            $generator(Bitboard::from_u64(1 << 60)),
            $generator(Bitboard::from_u64(1 << 61)),
            $generator(Bitboard::from_u64(1 << 62)),
            $generator(Bitboard::from_u64(1 << 63)),
        ]
    };
}

const fn knight_moves(mask: Bitboard) -> Bitboard {
    let mask = mask.bitboard;
    Bitboard::from_u64(
        ((mask << 15) & !FILES[7].bitboard)
        | ((mask << 17) & !FILES[0].bitboard)
        | ((mask << 6) & !(FILES[6].bitboard | FILES[7].bitboard))
        | ((mask << 10) & !(FILES[0].bitboard | FILES[1].bitboard))
        | ((mask >> 10) & !(FILES[6].bitboard | FILES[7].bitboard))
        | ((mask >> 6) & !(FILES[0].bitboard | FILES[1].bitboard))
        | ((mask >> 17) & !FILES[7].bitboard)
        | ((mask >> 15) & !FILES[0].bitboard))
}

const fn king_moves(mask: Bitboard) -> Bitboard {
    let mask = mask.bitboard;
    Bitboard::from_u64(
        ((mask << 7) & !FILES[7].bitboard)
        | (mask << 8)
        | ((mask << 9) & !FILES[0].bitboard)
        | ((mask << 1) & !FILES[0].bitboard)
        | ((mask >> 1) & !FILES[7].bitboard)
        | ((mask >> 7) & !FILES[0].bitboard)
        | (mask >> 8)
        | ((mask >> 9) & !FILES[7].bitboard))
}

const fn pawn_attacks_white(mask: Bitboard) -> Bitboard {
    let mask = mask.bitboard;
    Bitboard::from_u64(
        ((mask << 7) & !FILES[7].bitboard)
        | ((mask << 9) & !FILES[0].bitboard))
}

const fn pawn_attacks_black(mask: Bitboard) -> Bitboard {
    let mask = mask.bitboard;
    Bitboard::from_u64(
        ((mask >> 7) & !FILES[0].bitboard)
        | ((mask >> 9) & !FILES[7].bitboard))
}

pub const KNIGHT_TARGETS: [Bitboard; 64] = fill_mask_table!(knight_moves);
pub const KING_TARGETS: [Bitboard; 64] = fill_mask_table!(king_moves);
pub const PAWN_TARGETS: [[Bitboard; 64]; 2] = [fill_mask_table!(pawn_attacks_white), fill_mask_table!(pawn_attacks_black)];

const fn fill_between_entry(mut a: usize, mut b: usize) -> Bitboard {
    if a == b {
        return Bitboard::EMPTY;
    }

    const A: usize = 0;
    const B: usize = 1;

    let mut result = 0;

    if a > b {
        let t = a;
        a = b;
        b = t;
    }

    let file = [a % 8, b % 8];
    let rank = [a / 8, b / 8];
    let diag1 = [rank[A] + file[A], rank[B] + file[B]];
    let diag2 = [7 + rank[A] - file[A], 7 + rank[B] - file[B]];

    if file[A] == file[B] {
        b -= 8;
        while a != b {
            a += 8;
            result |= 1u64 << a;
        }
    } else if rank[A] == rank[B] {
        b -= 1;
        while a != b {
            a += 1;
            result |= 1u64 << a;
        }
    } else if diag1[A] == diag1[B] {
        b -= 7;
        while a != b {
            a += 7;
            result |= 1u64 << a;
        }
    } else if diag2[A] == diag2[B] {
        b -= 9;
        while a != b {
            a += 9;
            result |= 1u64 << a;
        }
    }
    Bitboard::from_u64(result)
}

const fn fill_between_row(row: usize) -> [Bitboard; 64] {
    let mut result = [Bitboard::EMPTY; 64];
    seq!(col in 0..64 {
        result[col] = fill_between_entry(row, col);
    });
    result
}

const fn fill_between_table() -> [[Bitboard; 64]; 64] {
    let mut result = [[Bitboard::EMPTY; 64]; 64];
    seq!(row in 0..64 {
        result[row] = fill_between_row(row);
    });
    result
}

pub const BETWEEN: [[Bitboard; 64]; 64] = fill_between_table();
