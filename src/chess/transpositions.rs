use rand::RngCore;
use crate::chess::board::{BLACK, Board, WHITE};
use crate::chess::moves::Move;

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

fn generate_side_keys() -> SideKeys {
    let mut result = [0; 64];
    let mut rng = rand::thread_rng();
    for i in 0..64 {
        result[i] = rng.next_u64();
    }
    result
}

fn generate_piece_keys() -> PieceKeys {
    [generate_side_keys(), generate_side_keys()]
}

fn generate_key() -> u64 {
    rand::thread_rng().next_u64()
}

fn generate_en_passant_keys() -> EnPassantKeys {
    let mut result = [0; 8];
    let mut rng = rand::thread_rng();
    for i in 0..8 {
        result[i] = rng.next_u64();
    }
    result
}

fn generate_castle_keys() -> CastleKeys {
    [generate_key(), generate_key()]
}

impl Zobrist {
    pub fn new() -> Self {
        Self {
            keys_pawns: generate_piece_keys(),
            keys_knights: generate_piece_keys(),
            keys_bishops: generate_piece_keys(),
            keys_rooks: generate_piece_keys(),
            keys_queens: generate_piece_keys(),
            keys_kings: generate_piece_keys(),
            key_black_to_move: generate_key(),
            key_castle_kingside: generate_castle_keys(),
            key_castle_queenside: generate_castle_keys(),
            keys_en_passant: generate_en_passant_keys(),
        }
    }

    pub fn key(&self, board: &Board, castle_kingside: [bool; 2], castle_queenside: [bool; 2]) -> u64 {
        let mut key = 0u64;

        for side in [ WHITE, BLACK ] {
            for (mask, keys) in [
                (board.pawns[side], &self.keys_pawns[side]),
                (board.knights[side], &self.keys_knights[side]),
                (board.bishops[side], &self.keys_bishops[side]),
                (board.rooks[side], &self.keys_rooks[side]),
                (board.queens[side], &self.keys_queens[side]),
                (board.kings[side], &self.keys_kings[side]),
            ] {
                key ^= self.key_piece(mask, keys);
            }

            if castle_kingside[side] {
                key ^= self.key_castle_kingside[side];
            }

            if castle_queenside[side] {
                key ^= self.key_castle_queenside[side];
            }
        }

        if board.side_to_move() == BLACK {
            key ^= self.key_black_to_move;
        }

        if board.en_passant != 0 {
            key ^= self.keys_en_passant[board.en_passant.trailing_zeros() as usize % 8];
        }

        key
    }

    fn key_piece(&self, mut mask: u64, keys: &[u64; 64]) -> u64 {
        let mut key = 0u64;

        while mask != 0 {
            let idx = mask.trailing_zeros() as usize;
            key ^= keys[idx];
            mask ^= 1u64 << idx;
        }

        key
    }
}

#[derive(Clone, Copy)]
pub enum Score {
    Exact(i64),
    LowerBound(i64),
    UpperBound(i64),
}

#[derive(Clone, Copy)]
struct Entry {
    hash: u64,
    depth: usize,
    score: Score,
    m: Move,
}

impl Entry {
    fn new() -> Self {
        Self {
            hash: 0,
            depth: 0,
            score: Score::Exact(0),
            m: Move::new(),
        }
    }
}

const TRANSPOSITION_TABLE_SIZE: usize = 2 * 1024 * 1024;
const TRANSPOSITION_TABLE_LENGTH: usize = TRANSPOSITION_TABLE_SIZE / size_of::<Entry>();

pub struct Transpositions {
    scores: Box<[Entry; TRANSPOSITION_TABLE_LENGTH]>,
}

impl Transpositions {
    pub fn new() -> Self {
        Self {
            scores: Box::new([Entry::new(); TRANSPOSITION_TABLE_LENGTH]),
        }
    }

    pub fn get(&self, hash: u64, depth: usize, alpha: i64, beta: i64) -> Option<(i64, Move)> {
        let entry = self.scores[hash as usize % TRANSPOSITION_TABLE_LENGTH];
        if entry.hash != hash {
            return None;
        }

        match entry.score {
            Score::Exact(score) => Some((score, entry.m)),
            Score::LowerBound(score) if score <= alpha => Some((score, entry.m)),
            Score::UpperBound(score) if score >= beta => Some((score, entry.m)),
            _ => None,
        }
    }

    pub fn set(&mut self, hash: u64, depth: usize, score: Score, m: Move) {
        self.scores[hash as usize % TRANSPOSITION_TABLE_LENGTH] = Entry {
            hash,
            depth,
            score,
            m,
        }
    }
}
