use crate::chess::board::Board;
use crate::chess::moves::Move;

pub struct SearchResult {
    pub m: Move,
    pub score: i64,
    pub depth: u64,
}

pub struct Options {
    pub white_time: u64,
    pub black_time: u64,
    pub white_increment: u64,
    pub black_increment: u64,
}

impl Options {
    pub fn new() -> Self {
        Self {
            white_time: u64::MAX,
            black_time: u64::MAX,
            white_increment: 0,
            black_increment: 0,
        }
    }
}

impl SearchResult {
    pub fn new(m: Move, score: i64, depth: u64) -> Self {
        Self { m, score, depth }
    }
}

macro_rules! result {
    ($m:expr,$s:expr,$d:expr) => {
        SearchResult::new($m, $s, $d)
    };
}

pub trait Search: pimpl::SearchImpl {
    fn search(&mut self) -> SearchResult {
        self.search_impl()
    }
}

impl Search for Board {}

mod pimpl {
    use super::*;
    use crate::chess::moves_generation::MoveGenerator;
    use rand::prelude::*;

    pub trait SearchImpl {
        fn search_impl(&mut self) -> SearchResult;
    }

    impl SearchImpl for Board {
        fn search_impl(&mut self) -> SearchResult {
            let moves = self.generate_moves();
            let chosen = moves
                .choose(&mut thread_rng())
                .cloned()
                .unwrap_or(Move::from_idx(0, 0));
            result!(chosen, 0, 1)
        }
    }
}
