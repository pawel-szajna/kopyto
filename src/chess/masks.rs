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

pub const KNIGHT_TARGETS: [u64; 64] = [
    knight_moves(1 << 0),
    knight_moves(1 << 1),
    knight_moves(1 << 2),
    knight_moves(1 << 3),
    knight_moves(1 << 4),
    knight_moves(1 << 5),
    knight_moves(1 << 6),
    knight_moves(1 << 7),
    knight_moves(1 << 8),
    knight_moves(1 << 9),
    knight_moves(1 << 10),
    knight_moves(1 << 11),
    knight_moves(1 << 12),
    knight_moves(1 << 13),
    knight_moves(1 << 14),
    knight_moves(1 << 15),
    knight_moves(1 << 16),
    knight_moves(1 << 17),
    knight_moves(1 << 18),
    knight_moves(1 << 19),
    knight_moves(1 << 20),
    knight_moves(1 << 21),
    knight_moves(1 << 22),
    knight_moves(1 << 23),
    knight_moves(1 << 24),
    knight_moves(1 << 25),
    knight_moves(1 << 26),
    knight_moves(1 << 27),
    knight_moves(1 << 28),
    knight_moves(1 << 29),
    knight_moves(1 << 30),
    knight_moves(1 << 31),
    knight_moves(1 << 32),
    knight_moves(1 << 33),
    knight_moves(1 << 34),
    knight_moves(1 << 35),
    knight_moves(1 << 36),
    knight_moves(1 << 37),
    knight_moves(1 << 38),
    knight_moves(1 << 39),
    knight_moves(1 << 40),
    knight_moves(1 << 41),
    knight_moves(1 << 42),
    knight_moves(1 << 43),
    knight_moves(1 << 44),
    knight_moves(1 << 45),
    knight_moves(1 << 46),
    knight_moves(1 << 47),
    knight_moves(1 << 48),
    knight_moves(1 << 49),
    knight_moves(1 << 50),
    knight_moves(1 << 51),
    knight_moves(1 << 52),
    knight_moves(1 << 53),
    knight_moves(1 << 54),
    knight_moves(1 << 55),
    knight_moves(1 << 56),
    knight_moves(1 << 57),
    knight_moves(1 << 58),
    knight_moves(1 << 59),
    knight_moves(1 << 60),
    knight_moves(1 << 61),
    knight_moves(1 << 62),
    knight_moves(1 << 63),
];
