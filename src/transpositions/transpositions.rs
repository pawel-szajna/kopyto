use crate::search::Score;
use crate::transpositions::TableScore;
use crate::types::Move;

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
            score: TableScore::Unknown,
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
        assert!(desired_size > 0);
        let length = desired_size * 1048576 / size_of::<Entry>();
        Self {
            length,
            scores: vec![Entry::new(); length].into_boxed_slice(),
        }
    }

    pub fn clear(&mut self) {
        self.scores.fill(Entry::new());
    }

    pub fn usage(&self) -> usize {
        let elems = self
            .scores
            .iter()
            .filter(|e| !matches!(e.score, TableScore::Unknown))
            .count();
        elems * 1000 / self.length
    }

    #[inline(always)]
    fn get_entry(&self, hash: u64) -> &Entry {
        unsafe { self.scores.get_unchecked(self.idx(hash)) }
    }

    #[inline(always)]
    fn idx(&self, hash: u64) -> usize {
        hash as usize % self.length
    }

    pub fn get_move(&self, hash: u64) -> Option<Move> {
        let entry = self.get_entry(hash);
        match entry.hash == hash
            && match entry.score {
                TableScore::Unknown => false,
                _ => true,
            } {
            true => Some(entry.m),
            false => None,
        }
    }

    #[inline(always)]
    pub fn prefetch(&self, hash: u64) {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            let scores_ptr = self.scores.as_ptr().add(self.idx(hash)) as *const i8;
            core::arch::x86_64::_mm_prefetch::<{ core::arch::x86_64::_MM_HINT_T0 }>(scores_ptr);
        }
    }

    pub fn get(&self, hash: u64, depth: i16, alpha: Score, beta: Score) -> Option<Score> {
        let entry = self.get_entry(hash);
        if entry.hash != hash {
            return None;
        }
        if entry.depth < depth {
            return None;
        }

        match entry.score {
            TableScore::Exact(score) => Some(score),
            TableScore::AtLeast(score) if score >= beta => Some(score),
            TableScore::AtMost(score) if score <= alpha => Some(score),
            _ => None,
        }
    }

    pub fn set(&mut self, hash: u64, depth: i16, score: TableScore, m: Move) {
        let idx = self.idx(hash);
        let old = &self.scores[idx];
        if match old.hash == hash {
            true => {
                old.depth <= depth
                    && match score {
                        TableScore::Unknown => false,
                        TableScore::Exact(_) => true,
                        TableScore::AtLeast(score) => match old.score {
                            TableScore::Exact(_) => false,
                            TableScore::AtLeast(old_score) => score > old_score,
                            _ => true,
                        },
                        TableScore::AtMost(score) => match old.score {
                            TableScore::Exact(_) => false,
                            TableScore::AtMost(old_score) => score < old_score,
                            _ => true,
                        },
                    }
            }
            false => !matches!(score, TableScore::Unknown) && match old.score {
                TableScore::Exact(_) => old.depth < depth,
                _ => old.depth <= depth,
            },
        } {
            self.scores[idx] = Entry { hash, depth, score, m }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_tt_has_null_usage() {
        assert_eq!(Transpositions::new(1).usage(), 0);
    }

    #[test]
    fn empty_tt_returns_neither_moves_nor_scores() {
        let tt = Transpositions::new(1);
        assert_eq!(tt.get(2137, 4, 10, 20), None);
        assert_eq!(tt.get_move(1234), None);
    }

    #[test]
    fn storing_getting_and_replacing() {
        let mut tt = Transpositions::new(1);
        tt.set(2137, 5, TableScore::Exact(10), Move::from_uci("e2e4"));
        assert_eq!(tt.get_move(2137), Some(Move::from_uci("e2e4")));
        assert_eq!(tt.get(2137, 3, 0, 100), Some(10));
        assert_eq!(tt.get(2137, 5, 0, 100), Some(10));
        assert_eq!(tt.get(2137, 8, 0, 100), None);

        tt.set(2137, 4, TableScore::Exact(5), Move::from_uci("e2e3"));
        assert_eq!(tt.get_move(2137), Some(Move::from_uci("e2e4")));
        assert_eq!(tt.get(2137, 4, -10, 10), Some(10));

        tt.set(2137, 8, TableScore::Exact(0), Move::from_uci("d2d4"));
        assert_eq!(tt.get_move(2137), Some(Move::from_uci("d2d4")));
        assert_eq!(tt.get(2137, 4, -10, 10), Some(0));

        tt.set(2137, 10, TableScore::Unknown, Move::from_uci("a1a1"));
        assert_eq!(tt.get_move(2137), Some(Move::from_uci("d2d4")));
        assert_eq!(tt.get(2137, 4, -10, 10), Some(0));

        tt.set(1234, 10, TableScore::AtLeast(0), Move::from_uci("e2e4"));
        assert_eq!(tt.get(1234, 8, -10, 10), None);
        assert_eq!(tt.get(1234, 8, -10, -5), Some(0));

        tt.set(0, 5, TableScore::AtMost(0), Move::from_uci("e2e4"));
        assert_eq!(tt.get(0, 5, -10, 10), None);
        assert_eq!(tt.get(0, 5, 5, 10), Some(0));
    }
}
