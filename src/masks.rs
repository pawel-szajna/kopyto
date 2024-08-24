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

pub const ROOK_QUEENSIDE: [u64; 2] = [1u64, 1u64 << (7 * 8)];
pub const ROOK_KINGSIDE: [u64; 2] = [1u64 << 7, 1u64 << (7 * 8 + 7)];
pub const ROOK_CASTLED_QUEENSIDE: [u64; 2] = [1u64 << 3, 1u64 << (7 * 8 + 3)];
pub const ROOK_CASTLED_KINGSIDE: [u64; 2] = [1u64 << 5, 1u64 << (7 * 8 + 5)];
pub const CASTLE_QUEENSIDE: [u64; 2] = [1u64 << 2, 1u64 << (7 * 8 + 2)];
pub const CASTLE_KINGSIDE: [u64; 2] = [1u64 << 6, 1u64 << (7 * 8 + 6)];
pub const LAST_RANK: [u64; 2] = [RANKS[7], RANKS[0]];
pub const SECOND_RANK: [u64; 2] = [RANKS[1], RANKS[6]];
pub const EN_PASSANT_RANK: [u64; 2] = [RANKS[3], RANKS[4]];
