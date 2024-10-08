use crate::moves_generation::{Moves, Weights};
use crate::types::Move;

pub struct MoveList {
    moves: Moves,
    weights: Weights,
}

impl MoveList {
    pub fn is_empty(&self) -> bool {
        self.moves.is_empty()
    }

    #[allow(dead_code)]
    pub fn contains(&self, m: &Move) -> bool {
        self.moves.contains(m)
    }
}

pub struct MoveListIterator {
    list: MoveList,
    used: usize,
}

impl MoveList {
    pub fn new(moves: Vec<Move>, weights: Weights) -> Self {
        assert_eq!(moves.len(), weights.len());
        Self {
            moves,
            weights,
        }
    }
}

impl IntoIterator for MoveList {
    type Item = Move;
    type IntoIter = MoveListIterator;

    fn into_iter(self) -> Self::IntoIter {
        MoveListIterator { list: self, used: 0 }
    }
}

impl Iterator for MoveListIterator {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.used >= self.list.moves.len() {
            return None;
        }

        let mut max_value = i32::MIN;
        let mut max_idx = 0;
        for i in self.used..self.list.moves.len() {
            let weight = self.list.weights[i];
            if weight > max_value {
                max_idx = i;
                max_value = weight;
            }
        }

        let best_move = self.list.moves[max_idx];
        self.list.weights.swap(self.used, max_idx);
        self.list.moves.swap(self.used, max_idx);
        self.used += 1;

        Some(best_move)
    }
}
