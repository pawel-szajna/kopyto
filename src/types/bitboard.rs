use std::fmt::{Binary, Formatter};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, ShlAssign, Shr, ShrAssign};
use crate::types::Square;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Bitboard {
    pub(crate) bitboard: u64,
}

impl Bitboard {
    pub const EMPTY: Bitboard = Self::new();

    pub const fn new() -> Self {
        Self { bitboard: 0 }
    }

    pub fn from_coords(file: usize, rank: usize) -> Self {
        Self::from(Square::from_coords(file, rank))
    }

    pub const fn from_u64(bitboard: u64) -> Self {
        Self { bitboard }
    }

    pub fn empty(&self) -> bool {
        self.bitboard == 0
    }

    pub fn not_empty(&self) -> bool {
        !self.empty()
    }

    pub fn peek(&self) -> Square {
        Square::from(self.bitboard.trailing_zeros() as usize)
    }

    pub fn pop(&mut self) -> Square {
        let result = self.peek();
        self.bitboard &= self.bitboard - 1;
        result
    }

    pub fn pieces(&self) -> u32 {
        self.bitboard.count_ones()
    }
}

impl From<Square> for Bitboard {
    fn from(square: Square) -> Self {
        Self { bitboard: 1 << (square as usize) }
    }
}

impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self { bitboard: self.bitboard & rhs.bitboard }
    }
}

impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.bitboard &= rhs.bitboard;
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self { bitboard: self.bitboard | rhs.bitboard }
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.bitboard |= rhs.bitboard;
    }
}

impl BitXor for Bitboard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self { bitboard: self.bitboard ^ rhs.bitboard }
    }
}

impl BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.bitboard ^= rhs.bitboard;
    }
}

impl Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self { bitboard: !self.bitboard }
    }
}

impl Binary for Bitboard {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.bitboard.fmt(f)
    }
}

impl<T> Shl<T> for Bitboard where u64: Shl<T>, <u64 as Shl<T>>::Output: Into<u64> {
    type Output = Self;

    fn shl(self, rhs: T) -> Self::Output {
        Self { bitboard: (self.bitboard << rhs).into() }
    }
}

impl<T> ShlAssign<T> for Bitboard where u64: ShlAssign<T> {
    fn shl_assign(&mut self, rhs: T) {
        self.bitboard <<= rhs;
    }
}

impl<T> Shr<T> for Bitboard where u64: Shr<T>, <u64 as Shr<T>>::Output: Into<u64> {
    type Output = Self;

    fn shr(self, rhs: T) -> Self::Output {
        Self { bitboard: (self.bitboard >> rhs).into() }
    }
}

impl<T> ShrAssign<T> for Bitboard where u64: ShrAssign<T> {
    fn shr_assign(&mut self, rhs: T) {
        self.bitboard >>= rhs;
    }
}

impl Iterator for Bitboard {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        match self.empty() {
            true => None,
            false => Some(self.pop()),
        }
    }
}
