use crate::types::Move;
use crate::transpositions::Score;

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
