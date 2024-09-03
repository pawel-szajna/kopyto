use rand::RngCore;
use crate::chess::board::Board;
use crate::chess::moves::Move;
use crate::chess::types::Side;

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
            mask &= mask - 1;
        }

        key
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Score {
    Exact(i64),
    LowerBound(i64),
    UpperBound(i64),
}

impl Score {
    pub fn from_alpha(alpha: i64, is_exact: bool) -> Self {
        match is_exact {
            true => Self::Exact(alpha),
            false => Self::UpperBound(alpha),
        }
    }
}

#[derive(Clone, Copy)]
struct Entry {
    hash: u64,
    depth: i64,
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

const TRANSPOSITION_TABLE_SIZE: usize = 64 * 1024 * 1024;
const TRANSPOSITION_TABLE_LENGTH: usize = TRANSPOSITION_TABLE_SIZE / size_of::<Entry>();

pub struct Transpositions {
    scores: Box<[Entry]>,
}

impl Transpositions {
    pub fn new() -> Self {
        Self {
            scores: vec![Entry::new(); TRANSPOSITION_TABLE_LENGTH].into_boxed_slice(),
        }
    }

    pub fn usage(&self) -> usize {
        let elems = self.scores.iter().filter(|e| e.hash != 0).count();
        elems * 1000 / TRANSPOSITION_TABLE_LENGTH
    }

    #[allow(dead_code)] // TODO decide what to do
    pub fn clear(&mut self) {
        self.scores.iter_mut().for_each(|x| *x = Entry::new());
    }

    pub fn get_move(&self, hash: u64) -> Option<Move> {
        let entry = self.scores[hash as usize % TRANSPOSITION_TABLE_LENGTH];
        match entry.hash == hash {
            true => Some(entry.m),
            false => None,
        }
    }

    pub fn get(&self, hash: u64, depth: i64, alpha: i64, beta: i64) -> Option<(i64, Move)> {
        let entry = self.scores[hash as usize % TRANSPOSITION_TABLE_LENGTH];
        if entry.hash != hash || entry.depth < depth {
            return None;
        }

        match entry.score {
            Score::Exact(score) => Some((score, entry.m)),
            Score::LowerBound(score) if score <= alpha => Some((score, entry.m)),
            Score::UpperBound(score) if score >= beta => Some((score, entry.m)),
            _ => None,
        }
    }

    pub fn set(&mut self, hash: u64, depth: i64, score: Score, m: Move) {
        let idx = hash as usize % TRANSPOSITION_TABLE_LENGTH;
        if self.scores[idx].hash != hash || self.scores[idx].depth <= depth {
            self.scores[hash as usize % TRANSPOSITION_TABLE_LENGTH] = Entry {
                hash,
                depth,
                score,
                m,
            }
        }
    }
}
