pub const SINGLE_RANK: u64 = 0b11111111;
pub const SINGLE_FILE: u64 = 0x0101010101010101;

pub const RANKS: [u64; 8] = [
    SINGLE_RANK << (8 * 0),
    SINGLE_RANK << (8 * 1),
    SINGLE_RANK << (8 * 2),
    SINGLE_RANK << (8 * 3),
    SINGLE_RANK << (8 * 4),
    SINGLE_RANK << (8 * 5),
    SINGLE_RANK << (8 * 6),
    SINGLE_RANK << (8 * 7),
];

pub const RANKS_RELATIVE: [[u64; 2]; 8] = [
    [RANKS[0], RANKS[7]],
    [RANKS[1], RANKS[6]],
    [RANKS[2], RANKS[5]],
    [RANKS[3], RANKS[4]],
    [RANKS[4], RANKS[3]],
    [RANKS[5], RANKS[2]],
    [RANKS[6], RANKS[1]],
    [RANKS[7], RANKS[0]],
];

pub const FILES: [u64; 8] = [
    SINGLE_FILE << 0,
    SINGLE_FILE << 1,
    SINGLE_FILE << 2,
    SINGLE_FILE << 3,
    SINGLE_FILE << 4,
    SINGLE_FILE << 5,
    SINGLE_FILE << 6,
    SINGLE_FILE << 7,
];

const LAST_RANK_IDX: u64 = 7 * 8;

pub const ROOK_QUEENSIDE: [u64; 2] = [1u64, 1u64 << LAST_RANK_IDX];
pub const ROOK_KINGSIDE: [u64; 2] = [1u64 << 7, 1u64 << (LAST_RANK_IDX + 7)];
pub const ROOK_CASTLED_QUEENSIDE: [u64; 2] = [1u64 << 3, 1u64 << (LAST_RANK_IDX + 3)];
pub const ROOK_CASTLED_KINGSIDE: [u64; 2] = [1u64 << 5, 1u64 << (LAST_RANK_IDX + 5)];
pub const CASTLE_QUEENSIDE: [u64; 2] = [1u64 << 2, 1u64 << (LAST_RANK_IDX + 2)];
pub const CASTLE_QUEENSIDE_BLOCKER_QUEEN: [u64; 2] = [1u64 << 3, 1u64 << (LAST_RANK_IDX + 3)];
pub const CASTLE_QUEENSIDE_BLOCKER_KNIGHT: [u64; 2] = [1u64 << 1, 1u64 << (LAST_RANK_IDX + 1)];
pub const CASTLE_KINGSIDE: [u64; 2] = [1u64 << 6, 1u64 << (LAST_RANK_IDX + 6)];
pub const CASTLE_KINGSIDE_BLOCKER: [u64; 2] = [1u64 << 5, 1u64 << (LAST_RANK_IDX + 5)];
pub const LAST_RANK: [u64; 2] = [RANKS[7], RANKS[0]];
pub const SECOND_RANK: [u64; 2] = [RANKS[1], RANKS[6]];
pub const EN_PASSANT_RANK: [u64; 2] = [RANKS[3], RANKS[4]];

macro_rules! fill_mask_table {
    ($generator:ident) => {
        [
            $generator(1 << 0),
            $generator(1 << 1),
            $generator(1 << 2),
            $generator(1 << 3),
            $generator(1 << 4),
            $generator(1 << 5),
            $generator(1 << 6),
            $generator(1 << 7),
            $generator(1 << 8),
            $generator(1 << 9),
            $generator(1 << 10),
            $generator(1 << 11),
            $generator(1 << 12),
            $generator(1 << 13),
            $generator(1 << 14),
            $generator(1 << 15),
            $generator(1 << 16),
            $generator(1 << 17),
            $generator(1 << 18),
            $generator(1 << 19),
            $generator(1 << 20),
            $generator(1 << 21),
            $generator(1 << 22),
            $generator(1 << 23),
            $generator(1 << 24),
            $generator(1 << 25),
            $generator(1 << 26),
            $generator(1 << 27),
            $generator(1 << 28),
            $generator(1 << 29),
            $generator(1 << 30),
            $generator(1 << 31),
            $generator(1 << 32),
            $generator(1 << 33),
            $generator(1 << 34),
            $generator(1 << 35),
            $generator(1 << 36),
            $generator(1 << 37),
            $generator(1 << 38),
            $generator(1 << 39),
            $generator(1 << 40),
            $generator(1 << 41),
            $generator(1 << 42),
            $generator(1 << 43),
            $generator(1 << 44),
            $generator(1 << 45),
            $generator(1 << 46),
            $generator(1 << 47),
            $generator(1 << 48),
            $generator(1 << 49),
            $generator(1 << 50),
            $generator(1 << 51),
            $generator(1 << 52),
            $generator(1 << 53),
            $generator(1 << 54),
            $generator(1 << 55),
            $generator(1 << 56),
            $generator(1 << 57),
            $generator(1 << 58),
            $generator(1 << 59),
            $generator(1 << 60),
            $generator(1 << 61),
            $generator(1 << 62),
            $generator(1 << 63),
        ]
    };
}

const fn knight_moves(mask: u64) -> u64 {
    0x0 | ((mask << 15) & !FILES[7])
        | ((mask << 17) & !FILES[0])
        | ((mask << 6) & !(FILES[6] | FILES[7]))
        | ((mask << 10) & !(FILES[0] | FILES[1]))
        | ((mask >> 10) & !(FILES[6] | FILES[7]))
        | ((mask >> 6) & !(FILES[0] | FILES[1]))
        | ((mask >> 17) & !FILES[7])
        | ((mask >> 15) & !FILES[0])
}

const fn king_moves(mask: u64) -> u64 {
    0x0 | ((mask << 7) & !FILES[7])
        | (mask << 8)
        | ((mask << 9) & !FILES[0])
        | ((mask << 1) & !FILES[0])
        | ((mask >> 1) & !FILES[7])
        | ((mask >> 7) & !FILES[0])
        | (mask >> 8)
        | ((mask >> 9) & !FILES[7])
}

pub const KNIGHT_TARGETS: [u64; 64] = fill_mask_table!(knight_moves);
pub const KING_TARGETS: [u64; 64] = fill_mask_table!(king_moves);
