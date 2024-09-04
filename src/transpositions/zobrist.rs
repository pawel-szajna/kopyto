use rand::RngCore;
use crate::board::Board;
use crate::types::{Bitboard, Side};

type SideKeys = [u64; 64];
type PieceKeys = [SideKeys; 2];
type EnPassantKeys = [u64; 8];
type CastleKeys = [u64; 2];

pub struct Zobrist {
    keys_pawns: PieceKeys,
    keys_knights: PieceKeys,
    keys_bishops: PieceKeys,
    keys_rooks: PieceKeys,
    keys_queens: PieceKeys,
    keys_kings: PieceKeys,
    key_black_to_move: u64,
    key_castle_kingside: CastleKeys,
    key_castle_queenside: CastleKeys,
    keys_en_passant: EnPassantKeys,
}

mod init {
    use super::*;

    fn generate_side_keys() -> SideKeys {
        let mut result = [0; 64];
        let mut rng = rand::thread_rng();
        for i in 0..64 {
            result[i] = rng.next_u64();
        }
        result
    }

    pub fn generate_piece_keys() -> PieceKeys {
        [generate_side_keys(), generate_side_keys()]
    }

    pub fn generate_key() -> u64 {
        rand::thread_rng().next_u64()
    }

    pub fn generate_en_passant_keys() -> EnPassantKeys {
        let mut result = [0; 8];
        let mut rng = rand::thread_rng();
        for i in 0..8 {
            result[i] = rng.next_u64();
        }
        result
    }

    pub fn generate_castle_keys() -> CastleKeys {
        [generate_key(), generate_key()]
    }
}

impl Zobrist {
    pub fn new() -> Self {
        Self {
            keys_pawns: init::generate_piece_keys(),
            keys_knights: init::generate_piece_keys(),
            keys_bishops: init::generate_piece_keys(),
            keys_rooks: init::generate_piece_keys(),
            keys_queens: init::generate_piece_keys(),
            keys_kings: init::generate_piece_keys(),
            key_black_to_move: init::generate_key(),
            key_castle_kingside: init::generate_castle_keys(),
            key_castle_queenside: init::generate_castle_keys(),
            keys_en_passant: init::generate_en_passant_keys(),
        }
    }

    pub fn key(&self, board: &Board, castle_kingside: [bool; 2], castle_queenside: [bool; 2]) -> u64 {
        let mut key = 0u64;

        key ^= self.key_piece(board.pawns[Side::White], &self.keys_pawns[Side::White]);
        key ^= self.key_piece(board.pawns[Side::Black], &self.keys_pawns[Side::Black]);
        key ^= self.key_piece(board.knights[Side::White], &self.keys_knights[Side::White]);
        key ^= self.key_piece(board.knights[Side::Black], &self.keys_knights[Side::Black]);
        key ^= self.key_piece(board.bishops[Side::White], &self.keys_bishops[Side::White]);
        key ^= self.key_piece(board.bishops[Side::Black], &self.keys_bishops[Side::Black]);
        key ^= self.key_piece(board.rooks[Side::White], &self.keys_rooks[Side::White]);
        key ^= self.key_piece(board.rooks[Side::Black], &self.keys_rooks[Side::Black]);
        key ^= self.key_piece(board.queens[Side::White], &self.keys_queens[Side::White]);
        key ^= self.key_piece(board.queens[Side::Black], &self.keys_queens[Side::Black]);
        key ^= self.key_piece(board.kings[Side::White], &self.keys_kings[Side::White]);
        key ^= self.key_piece(board.kings[Side::Black], &self.keys_kings[Side::Black]);

        if castle_kingside[Side::White] {
            key ^= self.key_castle_kingside[Side::White];
        }

        if castle_kingside[Side::Black] {
            key ^= self.key_castle_kingside[Side::Black];
        }

        if castle_queenside[Side::White] {
            key ^= self.key_castle_queenside[Side::White];
        }

        if castle_queenside[Side::Black] {
            key ^= self.key_castle_queenside[Side::Black];
        }

        if board.side_to_move().is_black() {
            key ^= self.key_black_to_move;
        }

        if board.en_passant.not_empty() {
            key ^= self.keys_en_passant[board.en_passant.peek().file()];
        }

        key
    }

    fn key_piece(&self, mask: Bitboard, keys: &[u64; 64]) -> u64 {
        let mut key = 0u64;

        for idx in mask {
            key ^= keys[idx];
        }

        key
    }
}
