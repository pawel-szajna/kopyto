use crate::search::Score;
use crate::types::Move;
use crate::transpositions::TableScore;

#[derive(Clone, Copy)]
struct Entry {
    hash: u64,
    depth: i16,
    score: TableScore,
    m: Move,
}

impl Entry {
    fn new() -> Self {
        Self {
            hash: 0,
            depth: 0,
            score: TableScore::Exact(0),
            m: Move::new(),
        }
    }
}

pub struct Transpositions {
    length: usize,
    scores: Box<[Entry]>,
}

impl Transpositions {
    pub fn new(desired_size: usize) -> Self {
        let length = desired_size * 1048576 / size_of::<Entry>();
        Self {
            length,
            scores: vec![Entry::new(); length].into_boxed_slice(),
        }
    }

    pub fn usage(&self) -> usize {
        let elems = self.scores.iter().filter(|e| e.hash != 0).count();
        elems * 1000 / self.length
    }

    pub fn get_move(&self, hash: u64) -> Option<Move> {
        let entry = self.scores[hash as usize % self.length];
        match entry.hash == hash {
            true => Some(entry.m),
            false => None,
        }
    }

    pub fn get(&self, hash: u64, depth: i16, alpha: Score, beta: Score) -> Option<Score> {
        let entry = self.scores[hash as usize % self.length];
        if entry.depth < depth || entry.hash != hash {
            return None;
        }

        match entry.score {
            TableScore::Exact(score) => Some(score),
            TableScore::LowerBound(score) if score <= alpha => Some(score),
            TableScore::UpperBound(score) if score >= beta => Some(score),
            _ => None,
        }
    }

    pub fn set(&mut self, hash: u64, depth: i16, score: TableScore, m: Move) {
        let idx = hash as usize % self.length;
        if self.scores[idx].hash != hash || self.scores[idx].depth <= depth {
            self.scores[hash as usize % self.length] = Entry {
                hash,
                depth,
                score,
                m,
            }
        }
    }
}
