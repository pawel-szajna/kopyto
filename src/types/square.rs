use std::fmt::{Display, Formatter};
use std::ops::{Index, IndexMut};

#[repr(usize)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Square {
    A1 = 0, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

impl Square {
    pub fn from_coords(file: usize, rank: usize) -> Self {
        Self::from(rank * 8 + file)
    }

    pub fn file(&self) -> usize {
        (*self as usize) % 8
    }

    pub fn rank(&self) -> usize {
        (*self as usize) / 8
    }

    pub fn north(&self) -> Self {
        Self::from(*self as usize + 8)
    }

    pub fn south(&self) -> Self {
        Self::from(*self as usize - 8)
    }

    pub fn east(&self) -> Self {
        Self::from(*self as usize + 1)
    }

    pub fn west(&self) -> Self {
        Self::from(*self as usize - 1)
    }

    pub fn northeast(&self) -> Self {
        Self::from(*self as usize + 9)
    }

    pub fn northwest(&self) -> Self {
        Self::from(*self as usize + 7)
    }

    pub fn southeast(&self) -> Self {
        Self::from(*self as usize - 7)
    }

    pub fn southwest(&self) -> Self {
        Self::from(*self as usize - 9)
    }

    pub fn is_white(&self) -> bool {
        (*self as usize % 2) != ((*self as usize / 8) % 2)
    }
}

impl<T> Index<Square> for [T; 64] {
    type Output = T;

    fn index(&self, index: Square) -> &Self::Output {
        &self[index as usize]
    }
}

impl<T> IndexMut<Square> for [T; 64] {
    fn index_mut(&mut self, index: Square) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

impl From<usize> for Square {
    fn from(value: usize) -> Self {
        unsafe { std::mem::transmute::<usize, Square>(value) }
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Square::A1 => "a1",
            Square::A2 => "a2",
            Square::A3 => "a3",
            Square::A4 => "a4",
            Square::A5 => "a5",
            Square::A6 => "a6",
            Square::A7 => "a7",
            Square::A8 => "a8",
            Square::B1 => "b1",
            Square::B2 => "b2",
            Square::B3 => "b3",
            Square::B4 => "b4",
            Square::B5 => "b5",
            Square::B6 => "b6",
            Square::B7 => "b7",
            Square::B8 => "b8",
            Square::C1 => "c1",
            Square::C2 => "c2",
            Square::C3 => "c3",
            Square::C4 => "c4",
            Square::C5 => "c5",
            Square::C6 => "c6",
            Square::C7 => "c7",
            Square::C8 => "c8",
            Square::D1 => "d1",
            Square::D2 => "d2",
            Square::D3 => "d3",
            Square::D4 => "d4",
            Square::D5 => "d5",
            Square::D6 => "d6",
            Square::D7 => "d7",
            Square::D8 => "d8",
            Square::E1 => "e1",
            Square::E2 => "e2",
            Square::E3 => "e3",
            Square::E4 => "e4",
            Square::E5 => "e5",
            Square::E6 => "e6",
            Square::E7 => "e7",
            Square::E8 => "e8",
            Square::F1 => "f1",
            Square::F2 => "f2",
            Square::F3 => "f3",
            Square::F4 => "f4",
            Square::F5 => "f5",
            Square::F6 => "f6",
            Square::F7 => "f7",
            Square::F8 => "f8",
            Square::G1 => "g1",
            Square::G2 => "g2",
            Square::G3 => "g3",
            Square::G4 => "g4",
            Square::G5 => "g5",
            Square::G6 => "g6",
            Square::G7 => "g7",
            Square::G8 => "g8",
            Square::H1 => "h1",
            Square::H2 => "h2",
            Square::H3 => "h3",
            Square::H4 => "h4",
            Square::H5 => "h5",
            Square::H6 => "h6",
            Square::H7 => "h7",
            Square::H8 => "h8",
        })
    }
}
