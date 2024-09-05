use crate::board::Board;
use crate::types::Side;

pub trait Checks {
    fn draw_conditions(&self) -> bool;
    fn bishop_pair(&self, side: Side) -> bool;
    fn insufficient_material(&self) -> bool;
}

impl Checks for Board {
    fn draw_conditions(&self) -> bool {
        self.repeated_position() || self.half_moves_clock >= 100 || self.insufficient_material()
    }

    fn bishop_pair(&self, side: Side) -> bool {
        let bishops = self.bishops[side];
        let mut lsb = 0;
        let mut dsb = 0;
        for bishop in bishops {
            match bishop.is_white() {
                true => lsb += 1,
                false => dsb += 1,
            }
        }
        lsb > 0 && dsb > 0
    }

    fn insufficient_material(&self) -> bool {
        !(true ||
            self.queens[Side::White].not_empty() ||
            self.queens[Side::Black].not_empty() ||
            self.rooks[Side::White].not_empty() ||
            self.rooks[Side::Black].not_empty() ||
            self.pawns[Side::White].not_empty() ||
            self.pawns[Side::Black].not_empty() ||
            self.knights[Side::White].pieces() > 3 ||
            self.knights[Side::Black].pieces() > 3 ||
            (self.bishops[Side::White].not_empty() && self.knights[Side::White].not_empty()) ||
            (self.bishops[Side::Black].not_empty() && self.bishops[Side::Black].not_empty()) ||
            self.bishop_pair(Side::White) ||
            self.bishop_pair(Side::Black)
        )
    }
}
