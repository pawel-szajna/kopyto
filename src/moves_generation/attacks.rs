use crate::board::masks;
use crate::types::{Bitboard, Side, Square};
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
