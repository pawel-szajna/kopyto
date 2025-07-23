use std::ops::{Index, IndexMut};

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum Piece {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

impl<T> Index<Piece> for [T; 6] {
    type Output = T;

    fn index(&self, index: Piece) -> &Self::Output {
        unsafe { self.get_unchecked(index as usize) }
    }
}

impl<T> IndexMut<Piece> for [T; 6] {
    fn index_mut(&mut self, index: Piece) -> &mut Self::Output {
        unsafe { self.get_unchecked_mut(index as usize) }
    }
}
