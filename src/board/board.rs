use crate::{masks, transpositions};
use crate::moves_generation;
use crate::types::{Bitboard, Move, Piece, Side};

pub type ColorBitboard = [Bitboard; 2];
pub type ColorBool = [bool; 2];
pub type PieceList = [Option<Piece>; 64];
pub type ColorPieceList = [PieceList; 2];

#[derive(Clone)]
struct History {
    from: Bitboard,
    to: Bitboard,
    castle_kingside: ColorBool,
    castle_queenside: ColorBool,
    capture: Option<Piece>,
    half_moves: u32,
    promotion: bool,
    en_passant: Bitboard,
    attacks: [Option<Bitboard>; 2],
    check: Option<bool>,
    checkmate: Option<bool>,
    hash: u64,
}

impl History {
    pub fn new(
        from: Bitboard,
        to: Bitboard,
        castle_kingside: ColorBool,
        castle_queenside: ColorBool,
        half_moves: u32,
        en_passant: Bitboard,
        check: Option<bool>,
        checkmate: Option<bool>,
        hash: u64,
    ) -> Self {
        Self {
            from,
            to,
            castle_kingside,
            castle_queenside,
            capture: None,
            half_moves,
            promotion: false,
            en_passant,
            attacks: [None, None],
            check,
            checkmate,
            hash,
        }
    }
}

#[derive(Clone)]
pub struct Board {
    pub kings: ColorBitboard,
    pub queens: ColorBitboard,
    pub rooks: ColorBitboard,
    pub bishops: ColorBitboard,
    pub knights: ColorBitboard,
    pub pawns: ColorBitboard,
    pub pieces: ColorPieceList,

    pub occupied: ColorBitboard,
    pub any_piece: Bitboard,

    pub castle_kingside: ColorBool,
    pub castle_queenside: ColorBool,

    pub current_color: Side,

    history: Vec<History>,
    hash: u64,

    pub half_moves_clock: u32,
    pub full_moves_count: u32,

    pub en_passant: Bitboard,
    check: Option<bool>,
    checkmate: Option<bool>,
    pub attacks: [Option<Bitboard>; 2],
    pub moves: [Option<Vec<Move>>; 2],
}

impl Board {
    pub fn new() -> Self {
        Self {
            kings: [Bitboard::EMPTY, Bitboard::EMPTY],
            queens: [Bitboard::EMPTY, Bitboard::EMPTY],
            rooks: [Bitboard::EMPTY, Bitboard::EMPTY],
            bishops: [Bitboard::EMPTY, Bitboard::EMPTY],
            knights: [Bitboard::EMPTY, Bitboard::EMPTY],
            pawns: [Bitboard::EMPTY, Bitboard::EMPTY],
            pieces: [[None; 64]; 2],

            occupied: [Bitboard::EMPTY, Bitboard::EMPTY],
            any_piece: Bitboard::EMPTY,

            castle_kingside: [true, true],
            castle_queenside: [true, true],

            current_color: Side::White,

            history: Vec::new(),
            hash: 0,

            half_moves_clock: 0,
            full_moves_count: 1,

            en_passant: Bitboard::EMPTY,
            check: None,
            checkmate: None,
            attacks: [None, None],
            moves: [None, None],
        }
    }

    pub fn from_starting_position() -> Board {
        let mut board = Board::new();

        board.put_king(Side::White, Bitboard::from_coords(4, 0));
        board.put_king(Side::Black, Bitboard::from_coords(4, 7));

        board.put_queen(Side::White, Bitboard::from_coords(3, 0));
        board.put_queen(Side::Black, Bitboard::from_coords(3, 7));

        board.put_rook(Side::White, Bitboard::from_coords(0, 0));
        board.put_rook(Side::White, Bitboard::from_coords(7, 0));
        board.put_rook(Side::Black, Bitboard::from_coords(0, 7));
        board.put_rook(Side::Black, Bitboard::from_coords(7, 7));

        board.put_bishop(Side::White, Bitboard::from_coords(2, 0));
        board.put_bishop(Side::White, Bitboard::from_coords(5, 0));
        board.put_bishop(Side::Black, Bitboard::from_coords(2, 7));
        board.put_bishop(Side::Black, Bitboard::from_coords(5, 7));

        board.put_knight(Side::White, Bitboard::from_coords(1, 0));
        board.put_knight(Side::White, Bitboard::from_coords(6, 0));
        board.put_knight(Side::Black, Bitboard::from_coords(1, 7));
        board.put_knight(Side::Black, Bitboard::from_coords(6, 7));

        for file in 0..8 {
            board.put_pawn(Side::White, Bitboard::from_coords(file, 1));
            board.put_pawn(Side::Black, Bitboard::from_coords(file, 6));
        }

        board.update_hash();
        board
    }

    fn put_piece_occupancy(&mut self, side: Side, mask: Bitboard) {
        self.occupied[side] |= mask;
        self.any_piece |= mask;
    }

    pub fn put_king(&mut self, side: Side, mask: Bitboard) {
        self.kings[side] |= mask;
        self.put_piece_occupancy(side, mask);
        self.pieces[side][mask.peek()] = Some(Piece::King);
    }

    pub fn put_queen(&mut self, side: Side, mask: Bitboard) {
        self.queens[side] |= mask;
        self.put_piece_occupancy(side, mask);
        self.pieces[side][mask.peek()] = Some(Piece::Queen);
    }

    pub fn put_rook(&mut self, side: Side, mask: Bitboard) {
        self.rooks[side] |= mask;
        self.put_piece_occupancy(side, mask);
        self.pieces[side][mask.peek()] = Some(Piece::Rook);
    }

    pub fn put_bishop(&mut self, side: Side, mask: Bitboard) {
        self.bishops[side] |= mask;
        self.put_piece_occupancy(side, mask);
        self.pieces[side][mask.peek()] = Some(Piece::Bishop);
    }

    pub fn put_knight(&mut self, side: Side, mask: Bitboard) {
        self.knights[side] |= mask;
        self.put_piece_occupancy(side, mask);
        self.pieces[side][mask.peek()] = Some(Piece::Knight);
    }

    pub fn put_pawn(&mut self, side: Side, mask: Bitboard) {
        self.pawns[side] |= mask;
        self.put_piece_occupancy(side, mask);
        self.pieces[side][mask.peek()] = Some(Piece::Pawn);
    }

    fn put_piece(&mut self, side: Side, mask: Bitboard, piece: Piece) {
        match piece {
            Piece::King => self.put_king(side, mask),
            Piece::Queen => self.put_queen(side, mask),
            Piece::Rook => self.put_rook(side, mask),
            Piece::Bishop => self.put_bishop(side, mask),
            Piece::Knight => self.put_knight(side, mask),
            Piece::Pawn => self.put_pawn(side, mask),
        }
    }

    fn remove_piece(&mut self, side: Side, mask: Bitboard) {
        let idx = mask.peek();
        self.any_piece ^= mask;
        self.occupied[side] ^= mask;
        match unsafe { self.pieces[side][idx].unwrap_unchecked() } {
            Piece::King => self.kings[side] ^= mask,
            Piece::Queen => self.queens[side] ^= mask,
            Piece::Rook => self.rooks[side] ^= mask,
            Piece::Bishop => self.bishops[side] ^= mask,
            Piece::Knight => self.knights[side] ^= mask,
            Piece::Pawn => self.pawns[side] ^= mask,
        }
        self.pieces[side][idx] = None;
    }

    pub fn has_piece(&self, mask: Bitboard) -> bool {
        !(self.any_piece & mask).empty()
    }

    pub fn get_attacks(&mut self, side: Side) -> Bitboard {
        match self.attacks[side] {
            Some(value) => value,
            None => {
                let attacks = moves_generation::real_attack_mask(self, side);
                self.attacks[side] = Some(attacks);
                attacks
            }
        }
    }

    pub fn in_check(&mut self) -> bool {
        match self.check {
            Some(value) => value,
            None => {
                let side = self.side_to_move();
                let opponent_attacks = self.get_attacks(!side);
                let is_in_check = (self.kings[side] & opponent_attacks).not_empty();
                self.check = Some(is_in_check);
                is_in_check
            }
        }
    }

    #[allow(dead_code)]
    pub fn in_checkmate(&mut self) -> bool {
        match self.checkmate {
            Some(value) => value,
            None => {
                let is_in_checkmate = match self.in_check() {
                    false => false,
                    true => moves_generation::generate_all(self).is_empty(),
                };
                self.checkmate = Some(is_in_checkmate);
                is_in_checkmate
            }
        }
    }

    pub fn check_piece(&self, side: Side, mask: Bitboard) -> Option<Piece> {
        self.pieces[side][mask.peek()]
    }

    pub fn key(&self) -> u64 {
        self.hash
    }

    fn check_side(&self, mask: Bitboard) -> Side {
        if (self.occupied[Side::White] & mask).not_empty() {
            return Side::White;
        }

        if (self.occupied[Side::Black] & mask).not_empty() {
            return Side::Black;
        }

        eprintln!("Board history:");
        for entry in &self.history {
            eprintln!("* {}{}", entry.from.peek(), entry.to.peek());
        }
        panic!("Internal error: there should be something on {} ({:#066b})", mask.peek(), mask);
    }

    pub fn update_hash(&mut self) {
        self.hash = transpositions::ZOBRIST.key(self, self.castle_kingside, self.castle_queenside);
    }

    pub fn make_null(&mut self) {
        let history_entry = History::new(
            Bitboard::EMPTY,
            Bitboard::EMPTY,
            self.castle_kingside,
            self.castle_queenside,
            self.half_moves_clock,
            self.en_passant,
            self.check,
            self.checkmate,
            self.hash,
        );

        self.history.push(history_entry);

        self.current_color = !self.current_color;

        if self.current_color.is_white() {
            self.full_moves_count += 1;
        }

        self.check = None;
        self.checkmate = None;
        self.attacks = [None, None];
        self.moves = [None, None];
        self.en_passant = Bitboard::EMPTY;
        self.half_moves_clock += 1;
        self.update_hash();
    }

    pub fn unmake_null(&mut self) {
        if self.current_color.is_white() {
            self.full_moves_count -= 1;
        }

        let history_entry = unsafe { self.history.pop().unwrap_unchecked() };

        self.half_moves_clock = history_entry.half_moves;
        self.current_color = !self.current_color;
        self.check = history_entry.check;
        self.checkmate = history_entry.checkmate;
        self.attacks = history_entry.attacks;
        self.moves = [None, None];
        self.en_passant = history_entry.en_passant;
        self.hash = history_entry.hash;
    }

    pub fn make_move(&mut self, m: Move) {
        let from_mask = Bitboard::from(m.get_from());
        let to_mask = Bitboard::from(m.get_to());

        let side = self.check_side(from_mask);
        let opponent = !side;

        let mut history_entry = History::new(
            from_mask,
            to_mask,
            self.castle_kingside,
            self.castle_queenside,
            self.half_moves_clock,
            self.en_passant,
            self.check,
            self.checkmate,
            self.hash,
        );

        if self.has_piece(to_mask) {
            history_entry.capture = self.check_piece(opponent, to_mask);
            self.remove_piece(opponent, to_mask);
        }

        let mut piece_type = unsafe { self.check_piece(side, from_mask).unwrap_unchecked() };

        if piece_type == Piece::Rook {
            if self.castle_queenside[side] && from_mask == masks::ROOK_QUEENSIDE[side] {
                self.castle_queenside[side] = false;
            } else if self.castle_kingside[side] && from_mask == masks::ROOK_KINGSIDE[side] {
                self.castle_kingside[side] = false;
            }
        }

        if piece_type == Piece::King {
            if to_mask == masks::CASTLE_KINGSIDE[side] && self.castle_kingside[side] {
                self.remove_piece(side, masks::ROOK_KINGSIDE[side]);
                self.put_piece(side, masks::ROOK_CASTLED_KINGSIDE[side], Piece::Rook);
            } else if to_mask == masks::CASTLE_QUEENSIDE[side] && self.castle_queenside[side] {
                self.remove_piece(side, masks::ROOK_QUEENSIDE[side]);
                self.put_piece(side, masks::ROOK_CASTLED_QUEENSIDE[side], Piece::Rook);
            }
            self.castle_queenside[side] = false;
            self.castle_kingside[side] = false;
        }

        if piece_type == Piece::Pawn || history_entry.capture.is_some() {
            self.half_moves_clock = 0;
        } else {
            self.half_moves_clock += 1;
        }

        if piece_type == Piece::Pawn && (to_mask & masks::LAST_RANK[side]).not_empty() {
            piece_type = Piece::from(m.get_promotion());
            history_entry.promotion = true;
        }

        if piece_type == Piece::Pawn && to_mask == self.en_passant {
            self.remove_piece(opponent, match side {
                Side::White => to_mask >> 8,
                Side::Black => to_mask << 8,
            });
        }

        self.en_passant = Bitboard::EMPTY;

        if piece_type == Piece::Pawn
            && (from_mask & masks::SECOND_RANK[side]).not_empty()
            && (to_mask & masks::EN_PASSANT_RANK[side]).not_empty()
            && ((to_mask << 1 | to_mask >> 1) & masks::EN_PASSANT_RANK[side] & self.pawns[opponent]).not_empty()
        {
            self.en_passant = match side {
                Side::White => from_mask << 8,
                Side::Black => from_mask >> 8,
            }
        }

        self.put_piece(side, to_mask, piece_type);
        self.remove_piece(side, from_mask);

        self.current_color = opponent;

        if self.current_color.is_white() {
            self.full_moves_count += 1;
        }

        history_entry.attacks = self.attacks;

        self.history.push(history_entry);
        self.check = None;
        self.checkmate = None;
        self.attacks = [None, None];
        self.moves = [None, None];
        self.update_hash();
    }

    pub fn unmake_move(&mut self) {
        if self.history.is_empty() {
            panic!("Cannot unmake move with no moves");
        }

        let last_move = unsafe { self.history.pop().unwrap_unchecked() };
        let side = self.check_side(last_move.to);
        let opponent = !side;

        if self.castle_kingside[side] != last_move.castle_kingside[side]
            && last_move.from == masks::KING_STARTING_POSITION[side]
            && last_move.to == masks::CASTLE_KINGSIDE[side]
            && self.check_piece(side, masks::CASTLE_KINGSIDE[side]) == Some(Piece::King)
        {
            self.remove_piece(side, masks::ROOK_CASTLED_KINGSIDE[side]);
            self.put_piece(side, masks::ROOK_KINGSIDE[side], Piece::Rook);
        }
        if self.castle_queenside[side] != last_move.castle_queenside[side]
            && last_move.from == masks::KING_STARTING_POSITION[side]
            && last_move.to == masks::CASTLE_QUEENSIDE[side]
            && self.check_piece(side, masks::CASTLE_QUEENSIDE[side]) == Some(Piece::King)
        {
            self.remove_piece(side, masks::ROOK_CASTLED_QUEENSIDE[side]);
            self.put_piece(side, masks::ROOK_QUEENSIDE[side], Piece::Rook);
        }

        self.castle_kingside = last_move.castle_kingside;
        self.castle_queenside = last_move.castle_queenside;

        let mut piece_type = unsafe {self.check_piece(side, last_move.to).unwrap_unchecked() };

        if last_move.promotion {
            piece_type = Piece::Pawn;
        }

        if piece_type == Piece::Pawn && last_move.en_passant == last_move.to {
            let capture_square = last_move.en_passant;
            self.put_piece(
                opponent,
                match side {
                    Side::White => capture_square >> 8,
                    Side::Black => capture_square << 8,
                },
                Piece::Pawn,
            );
        }

        self.remove_piece(side, last_move.to);
        self.put_piece(side, last_move.from, piece_type);

        if last_move.capture.is_some() {
            self.put_piece(opponent, last_move.to, unsafe { last_move.capture.unwrap_unchecked() });
        }

        self.current_color = side;
        self.half_moves_clock = last_move.half_moves;
        self.en_passant = last_move.en_passant;
        if self.current_color.is_black() {
            self.full_moves_count -= 1;
        }
        self.check = last_move.check;
        self.checkmate = last_move.checkmate;
        self.attacks = last_move.attacks;
        self.moves = [None, None];
        self.hash = last_move.hash;
    }

    pub fn repeated_position(&self) -> bool {
        self.history.iter().any(|h| h.hash == self.hash)
    }

    pub fn side_to_move(&self) -> Side {
        self.current_color
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{FenConsumer, FenProducer};
    use crate::types::Promotion;

    impl Board {
        fn make_move_str(&mut self, from: &str, to: &str) {
            self.make_move(Move::from_str(from, to));
        }

        fn assert_position(&self, fen: &str) {
            let actual = self.export_fen();
            assert_eq!(actual, fen);
        }
    }

    #[test]
    fn test_starting_position() {
        Board::from_starting_position().assert_position("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    }

    #[test]
    fn test_capture_moves() {
        let mut board = Board::from_fen("r2qkbnr/ppp1pppp/2n5/3p1b2/1P2P3/2N5/P1PP1PPP/R1BQKBNR w KQkq - 1 4");
        board.make_move_str("e4", "f5");
        board.assert_position("r2qkbnr/ppp1pppp/2n5/3p1P2/1P6/2N5/P1PP1PPP/R1BQKBNR b KQkq - 0 4");
        board.make_move_str("c6", "b4");
        board.assert_position("r2qkbnr/ppp1pppp/8/3p1P2/1n6/2N5/P1PP1PPP/R1BQKBNR w KQkq - 0 5");
        board.make_move_str("c3", "d5");
        board.assert_position("r2qkbnr/ppp1pppp/8/3N1P2/1n6/8/P1PP1PPP/R1BQKBNR b KQkq - 0 5");
        board.make_move_str("d8", "d5");
        board.assert_position("r3kbnr/ppp1pppp/8/3q1P2/1n6/8/P1PP1PPP/R1BQKBNR w KQkq - 0 6");
        board.unmake_move();
        board.assert_position("r2qkbnr/ppp1pppp/8/3N1P2/1n6/8/P1PP1PPP/R1BQKBNR b KQkq - 0 5");
        board.unmake_move();
        board.assert_position("r2qkbnr/ppp1pppp/8/3p1P2/1n6/2N5/P1PP1PPP/R1BQKBNR w KQkq - 0 5");
        board.unmake_move();
        board.assert_position("r2qkbnr/ppp1pppp/2n5/3p1P2/1P6/2N5/P1PP1PPP/R1BQKBNR b KQkq - 0 4");
        board.unmake_move();
        board.assert_position("r2qkbnr/ppp1pppp/2n5/3p1b2/1P2P3/2N5/P1PP1PPP/R1BQKBNR w KQkq - 1 4");
    }

    #[test]
    fn test_castle_moves() {
        let mut board = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
        board.make_move_str("e1", "g1");
        board.assert_position("r3k2r/8/8/8/8/8/8/R4RK1 b kq - 1 1");
        board.make_move_str("h8", "h2");
        board.assert_position("r3k3/8/8/8/8/8/7r/R4RK1 w q - 2 2");
        board.make_move_str("f1", "f2");
        board.assert_position("r3k3/8/8/8/8/8/5R1r/R5K1 b q - 3 2");
        board.make_move_str("e8", "c8");
        board.assert_position("2kr4/8/8/8/8/8/5R1r/R5K1 w - - 4 3");
        board.unmake_move();
        board.assert_position("r3k3/8/8/8/8/8/5R1r/R5K1 b q - 3 2");
        board.unmake_move();
        board.assert_position("r3k3/8/8/8/8/8/7r/R4RK1 w q - 2 2");
        board.unmake_move();
        board.assert_position("r3k2r/8/8/8/8/8/8/R4RK1 b kq - 1 1");
        board.unmake_move();
        board.assert_position("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
    }

    #[test]
    fn test_promotion() {
        let mut board = Board::from_fen("rnbqkbnr/p1pppppP/8/8/8/8/PPpPPP1P/RNBQKBNR w KQkq - 0 5");
        board.make_move(Move::from_str_prom("h7", "g8", Promotion::Queen));
        board.assert_position("rnbqkbQr/p1ppppp1/8/8/8/8/PPpPPP1P/RNBQKBNR b KQkq - 0 5");
        board.make_move(Move::from_str_prom("c2", "d1", Promotion::Knight));
        board.assert_position("rnbqkbQr/p1ppppp1/8/8/8/8/PP1PPP1P/RNBnKBNR w KQkq - 0 6");
        board.unmake_move();
        board.assert_position("rnbqkbQr/p1ppppp1/8/8/8/8/PPpPPP1P/RNBQKBNR b KQkq - 0 5");
        board.unmake_move();
        board.assert_position("rnbqkbnr/p1pppppP/8/8/8/8/PPpPPP1P/RNBQKBNR w KQkq - 0 5");
    }

    #[test]
    fn test_en_passant() {
        let mut board = Board::from_fen("rnbqkbnr/pppp1pp1/7p/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq e6 0 3");
        board.make_move_str("d5", "e6");
        board.assert_position("rnbqkbnr/pppp1pp1/4P2p/8/8/8/PPP1PPPP/RNBQKBNR b KQkq - 0 3");
        board.make_move_str("d7", "d5");
        board.assert_position("rnbqkbnr/ppp2pp1/4P2p/3p4/8/8/PPP1PPPP/RNBQKBNR w KQkq - 0 4");
        board.make_move_str("e6", "e7");
        board.assert_position("rnbqkbnr/ppp1Ppp1/7p/3p4/8/8/PPP1PPPP/RNBQKBNR b KQkq - 0 4");
        board.make_move_str("d5", "d4");
        board.assert_position("rnbqkbnr/ppp1Ppp1/7p/8/3p4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 5");
        board.make_move_str("e2", "e4");
        board.assert_position("rnbqkbnr/ppp1Ppp1/7p/8/3pP3/8/PPP2PPP/RNBQKBNR b KQkq e3 0 5");
        board.make_move_str("d4", "e3");
        board.assert_position("rnbqkbnr/ppp1Ppp1/7p/8/8/4p3/PPP2PPP/RNBQKBNR w KQkq - 0 6");
        board.unmake_move();
        board.assert_position("rnbqkbnr/ppp1Ppp1/7p/8/3pP3/8/PPP2PPP/RNBQKBNR b KQkq e3 0 5");
        board.unmake_move();
        board.assert_position("rnbqkbnr/ppp1Ppp1/7p/8/3p4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 5");
        board.unmake_move();
        board.assert_position("rnbqkbnr/ppp1Ppp1/7p/3p4/8/8/PPP1PPPP/RNBQKBNR b KQkq - 0 4");
        board.unmake_move();
        board.assert_position("rnbqkbnr/ppp2pp1/4P2p/3p4/8/8/PPP1PPPP/RNBQKBNR w KQkq - 0 4");
        board.unmake_move();
        board.assert_position("rnbqkbnr/pppp1pp1/4P2p/8/8/8/PPP1PPPP/RNBQKBNR b KQkq - 0 3");
        board.unmake_move();
        board.assert_position("rnbqkbnr/pppp1pp1/7p/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq e6 0 3");
    }

    mod bugs {
        use super::*;

        impl Board {
            fn uci(&mut self, m: &str) {
                self.make_move(Move::from_uci(m));
            }
        }

        #[test]
        fn bug_1() {
            // after: 1. Nf3 a6 2. Rg1 and undoing the last move, the bishop on f1 disappears
            // (bug in handling castling during unmake)
            let mut board = Board::from_starting_position();
            board.uci("g1f3");
            board.uci("a7a6");
            board.assert_position("rnbqkbnr/1ppppppp/p7/8/8/5N2/PPPPPPPP/RNBQKB1R w KQkq - 0 2");
            board.uci("h1g1");
            board.assert_position("rnbqkbnr/1ppppppp/p7/8/8/5N2/PPPPPPPP/RNBQKBR1 b Qkq - 1 2");
            board.unmake_move();
            board.assert_position("rnbqkbnr/1ppppppp/p7/8/8/5N2/PPPPPPPP/RNBQKB1R w KQkq - 0 2");
        }

        #[test]
        fn bug_2() {
            // kiwipete e1d1 e8c8 d1c1 (undo) a1b1 crash
            let mut board = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
            board.uci("e1d1");
            board.assert_position("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R2K3R b kq - 1 1");
            board.uci("e8c8");
            board.assert_position("2kr3r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R2K3R w - - 2 2");
            board.uci("d1c1");
            board.assert_position("2kr3r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R1K4R b - - 3 2");
            board.unmake_move();
            board.assert_position("2kr3r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R2K3R w - - 2 2");
            board.uci("a1b1");
            board.assert_position("2kr3r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/1R1K3R b - - 3 2");
        }

        #[test]
        fn bug_3() {
            // kiwipete e2d1 a6f1 (undo) crash
            let mut board = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
            board.uci("e2d1");
            board.assert_position("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPB1PPP/R2BK2R b KQkq - 1 1");
            board.uci("a6f1");
            board.assert_position("r3k2r/p1ppqpb1/1n2pnp1/3PN3/1p2P3/2N2Q1p/PPPB1PPP/R2BKb1R w KQkq - 2 2");
            board.unmake_move();
            board.assert_position("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPB1PPP/R2BK2R b KQkq - 1 1");
        }

        #[test]
        fn bug_4() {
            // r1bqkbnr/pppp2pp/8/4pp2/8/2NnP1PP/PPPPNP2/R1BQKB1R w KQkq - 4 3 should detect in check
            let mut board = Board::from_fen("r1bqkbnr/pppp2pp/8/4pp2/8/2NnP1PP/PPPPNP2/R1BQKB1R w KQkq - 4 3");
            assert_eq!(board.in_check(), true);
        }
    }
}
