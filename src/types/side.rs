use std::fmt::{Display, Formatter};
use std::ops::{Index, IndexMut, Not};

#[derive(Copy, Clone)]
#[repr(usize)]
pub enum Side {
    White = 0,
    Black = 1,
}

impl Display for Side {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Side::White => "White",
            Side::Black => "Black",
        })
    }
}

impl Not for Side {
    type Output = Side;

    fn not(self) -> Self::Output {
        self.opponent()
    }
}

impl<T> Index<Side> for [T; 2] {
    type Output = T;

    fn index(&self, index: Side) -> &Self::Output {
        &self[index as usize]
    }
}

impl<T> IndexMut<Side> for [T; 2] {
    fn index_mut(&mut self, index: Side) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

impl Side {
    pub fn opponent(&self) -> Side {
        match self {
            Side::White => Side::Black,
            Side::Black => Side::White,
        }
    }

    pub fn is_white(&self) -> bool {
        match self {
            Side::White => true,
            _ => false,
        }
    }

    pub fn is_black(&self) -> bool {
        !self.is_white()
    }

    pub fn choose<T>(&self, if_white: T, if_black: T) -> T {
        match self {
            Side::White => if_white,
            Side::Black => if_black,
        }
    }
}
