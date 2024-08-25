use super::util;
use std::fmt;

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum Piece {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

#[repr(u16)]
#[derive(Debug)]
pub enum Promotion {
    Queen = 0,
    Rook = 1,
    Bishop = 2,
    Knight = 3,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct Move {
    m: u16,
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let from = util::idx_to_str(self.get_from() as usize);
        let to = util::idx_to_str(self.get_to() as usize);
        match self.m & 0b100000000000000 != 0 {
            true => write!(f, "{}{}+{:?}", from, to, self.get_promotion()),
            false => write!(f, "{}{}", from, to),
        }
    }
}

impl From<u16> for Promotion {
    fn from(value: u16) -> Self {
        match value {
            0 => Promotion::Queen,
            1 => Promotion::Rook,
            2 => Promotion::Bishop,
            3 => Promotion::Knight,
            _ => panic!("Invalid promotion"),
        }
    }
}

impl From<Promotion> for Piece {
    fn from(value: Promotion) -> Self {
        match value {
            Promotion::Queen => Piece::Queen,
            Promotion::Rook => Piece::Rook,
            Promotion::Bishop => Piece::Bishop,
            Promotion::Knight => Piece::Knight,
        }
    }
}

impl Move {
    const MASK_FROM: u16 = 0b111111;
    const MASK_TO: u16 = 0b111111000000;
    const MASK_PROMOTION: u16 = 0b11000000000000;
    const MASK_HAS_PROMOTION: u16 = 0b100000000000000;

    pub fn new() -> Self {
        Self { m: 0 }
    }

    pub fn from_str(from: &str, to: &str) -> Self {
        Self::from_idx(util::str_to_idx(from), util::str_to_idx(to))
    }

    pub fn from_str_prom(from: &str, to: &str, promotion: Promotion) -> Self {
        let mut m = Self::from_str(from, to);
        m.set_promotion(promotion);
        m
    }

    pub fn from_uci(uci: &str) -> Self {
        match uci.len() {
            4 => Self::from_str(&uci[0..2], &uci[2..4]),
            5 => Self::from_str_prom(
                &uci[0..2],
                &uci[2..4],
                match &uci[4..5] {
                    "q" => Promotion::Queen,
                    "r" => Promotion::Rook,
                    "b" => Promotion::Bishop,
                    "n" => Promotion::Knight,
                    _ => panic!("invalid uci move: {} (bad promotion)", uci),
                },
            ),
            _ => panic!("invalid uci move: {}", uci),
        }
    }

    pub fn from_idx(from: usize, to: usize) -> Self {
        let mut m = Self::new();
        m.set_from(from);
        m.set_to(to);
        m
    }

    pub fn from_mask(from: u64, to: u64) -> Self {
        Self::from_idx(from.trailing_zeros() as usize, to.trailing_zeros() as usize)
    }

    pub fn from_idx_prom(from: usize, to: usize, promotion: Promotion) -> Self {
        let mut m = Self::from_idx(from, to);
        m.set_promotion(promotion);
        m
    }

    pub fn set_from(&mut self, from: usize) {
        self.m |= (from as u16) & Self::MASK_FROM;
    }

    pub fn set_to(&mut self, to: usize) {
        self.m |= ((to as u16) << 6) & Self::MASK_TO;
    }

    pub fn set_promotion(&mut self, promotion: Promotion) {
        self.m |= (((promotion as u16) << 12) & Self::MASK_PROMOTION) | Self::MASK_HAS_PROMOTION;
    }

    pub fn get_from(&self) -> u16 {
        self.m & Self::MASK_FROM
    }

    pub fn get_to(&self) -> u16 {
        (self.m & Self::MASK_TO) >> 6
    }

    pub fn get_promotion(&self) -> Promotion {
        Promotion::from((self.m & Self::MASK_PROMOTION) >> 12)
    }

    pub fn to_uci(&self) -> String {
        let from = util::idx_to_str(self.get_from() as usize);
        let to = util::idx_to_str(self.get_to() as usize);
        match self.m & 0b100000000000000 != 0 {
            false => format!("{}{}", from, to),
            true => {
                format!(
                    "{}{}{}",
                    from,
                    to,
                    match self.get_promotion() {
                        Promotion::Queen => 'q',
                        Promotion::Rook => 'r',
                        Promotion::Bishop => 'b',
                        Promotion::Knight => 'n',
                    }
                )
            }
        }
    }
}
