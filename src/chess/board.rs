use crate::chess::book::{Book, BookGenerator};
use crate::chess::magics::{create_bishop_magics, create_rook_magics, Magics};
use crate::chess::transpositions::{Transpositions, Zobrist};
use super::masks;
use super::moves::*;
use super::moves_generation::MoveGenerator;
use super::util::*;
use super::types::Side;

pub type Bitboard = u64;
pub type ColorBitboard = [Bitboard; 2];
pub type ColorBool = [bool; 2];

struct History {
    from: u64,
    to: u64,
    castle_kingside: ColorBool,
    castle_queenside: ColorBool,
    capture: Option<Piece>,
    half_moves: u32,
    promotion: bool,
    en_passant: u64,
    attacks: [Option<u64>; 2],
    check: [Option<bool>; 2],
    checkmate: [Option<bool>; 2],
    hash: u64,
}

impl History {
    pub fn new(
        from: u64,
        to: u64,
        castle_kingside: ColorBool,
        castle_queenside: ColorBool,
        half_moves: u32,
        en_passant: u64,
        check: [Option<bool>; 2],
        checkmate: [Option<bool>; 2],
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

pub struct Board {
    pub(super) kings: ColorBitboard,
    pub(super) queens: ColorBitboard,
    pub(super) rooks: ColorBitboard,
    pub(super) bishops: ColorBitboard,
    pub(super) knights: ColorBitboard,
    pub(super) pawns: ColorBitboard,

    pub(super) occupied: ColorBitboard,
    pub(super) any_piece: Bitboard,

    pub(super) castle_kingside: ColorBool,
    pub(super) castle_queenside: ColorBool,

    current_color: Side,

    history: Vec<History>,
    zobrist: Zobrist,
    pub(super) transpositions: Transpositions,
    hash: u64,

    pub(super) half_moves_clock: u32,
    pub(super) full_moves_count: u32,

    pub(super) en_passant: u64,
    check: [Option<bool>; 2],
    checkmate: [Option<bool>; 2],
    pub(super) attacks: [Option<u64>; 2],
    pub(super) moves: [Option<Vec<Move>>; 2],

    pub(super) last_eval: i64,

    pub(super) bishop_magics: Magics,
    pub(super) rook_magics: Magics,

    pub(super) book: Book,
}

impl Board {
    pub fn new() -> Self {
        Self {
            kings: [0, 0],
            queens: [0, 0],
            rooks: [0, 0],
            bishops: [0, 0],
            knights: [0, 0],
            pawns: [0, 0],

            occupied: [0, 0],
            any_piece: 0,

            castle_kingside: [true, true],
            castle_queenside: [true, true],

            current_color: Side::White,

            history: Vec::new(),
            zobrist: Zobrist::new(),
            transpositions: Transpositions::new(),
            hash: 0,

            half_moves_clock: 0,
            full_moves_count: 1,

            en_passant: 0,
            check: [None, None],
            checkmate: [None, None],
            attacks: [None, None],
            moves: [None, None],

            last_eval: 0,

            bishop_magics: create_bishop_magics(),
            rook_magics: create_rook_magics(),

            book: Book::new(),
        }
    }

    pub fn from_fen(fen: &str) -> Board {
        let mut board = Board::new();
        let mut fen = fen.chars();

        for rank in 0..8 {
            let mut file = 0;
            loop {
                match fen.next() {
                    None => return board,
                    Some('/') => break,
                    Some(' ') => break,
                    Some('2') => file += 1,
                    Some('3') => file += 2,
                    Some('4') => file += 3,
                    Some('5') => file += 4,
                    Some('6') => file += 5,
                    Some('7') => file += 6,
                    Some('8') => file += 7,
                    Some('k') => board.put_king(Side::Black, coords_to_mask(file, 7 - rank)),
                    Some('K') => board.put_king(Side::White, coords_to_mask(file, 7 - rank)),
                    Some('q') => board.put_queen(Side::Black, coords_to_mask(file, 7 - rank)),
                    Some('Q') => board.put_queen(Side::White, coords_to_mask(file, 7 - rank)),
                    Some('r') => board.put_rook(Side::Black, coords_to_mask(file, 7 - rank)),
                    Some('R') => board.put_rook(Side::White, coords_to_mask(file, 7 - rank)),
                    Some('p') => board.put_pawn(Side::Black, coords_to_mask(file, 7 - rank)),
                    Some('P') => board.put_pawn(Side::White, coords_to_mask(file, 7 - rank)),
                    Some('n') => board.put_knight(Side::Black, coords_to_mask(file, 7 - rank)),
                    Some('N') => board.put_knight(Side::White, coords_to_mask(file, 7 - rank)),
                    Some('b') => board.put_bishop(Side::Black, coords_to_mask(file, 7 - rank)),
                    Some('B') => board.put_bishop(Side::White, coords_to_mask(file, 7 - rank)),
                    _ => {}
                }
                file += 1;
            }
        }

        match fen.next() {
            Some('w') => board.current_color = Side::White,
            Some('b') => board.current_color = Side::Black,
            _ => panic!("Invalid fen, expected color to play"),
        }

        assert_eq!(fen.next(), Some(' '), "Invalid fen, expected space");

        board.castle_kingside = [false, false];
        board.castle_queenside = [false, false];

        loop {
            match fen.next() {
                Some('-') => {
                    fen.next();
                    break;
                }
                Some(' ') => break,
                Some('K') => board.castle_kingside[Side::White] = true,
                Some('k') => board.castle_kingside[Side::Black] = true,
                Some('Q') => board.castle_queenside[Side::White] = true,
                Some('q') => board.castle_queenside[Side::Black] = true,
                _ => panic!("Invalid fen, expected castling rights"),
            }
        }

        loop {
            match fen.next() {
                Some('-') => {
                    fen.next();
                    break;
                }
                Some(' ') => break,
                Some(file) if file.is_alphabetic() => {
                    let rank = fen.next().unwrap();
                    board.en_passant = 1u64 << str_to_idx(format!("{}{}", file, rank).as_str());
                }
                _ => panic!("Invalid fen, expected en passant data"),
            }
        }

        board.half_moves_clock = 0;

        loop {
            match fen.next() {
                Some(' ') => break,
                Some(x) if x.is_digit(10) => {
                    board.half_moves_clock = board.half_moves_clock * 10 + x.to_digit(10).unwrap()
                }
                _ => panic!("Invalid fen, expected half move count"),
            }
        }

        board.full_moves_count = 0;

        loop {
            match fen.next() {
                Some(x) if x.is_digit(10) => {
                    board.full_moves_count = board.full_moves_count * 10 + x.to_digit(10).unwrap()
                }
                _ => break,
            }
        }

        board.update_hash();
        board
    }

    pub fn from_starting_position() -> Board {
        let mut board = Board::new();

        board.put_king(Side::White, coords_to_mask(4, 0));
        board.put_king(Side::Black, coords_to_mask(4, 7));

        board.put_queen(Side::White, coords_to_mask(3, 0));
        board.put_queen(Side::Black, coords_to_mask(3, 7));

        board.put_rook(Side::White, coords_to_mask(0, 0));
        board.put_rook(Side::White, coords_to_mask(7, 0));
        board.put_rook(Side::Black, coords_to_mask(0, 7));
        board.put_rook(Side::Black, coords_to_mask(7, 7));

        board.put_bishop(Side::White, coords_to_mask(2, 0));
        board.put_bishop(Side::White, coords_to_mask(5, 0));
        board.put_bishop(Side::Black, coords_to_mask(2, 7));
        board.put_bishop(Side::Black, coords_to_mask(5, 7));

        board.put_knight(Side::White, coords_to_mask(1, 0));
        board.put_knight(Side::White, coords_to_mask(6, 0));
        board.put_knight(Side::Black, coords_to_mask(1, 7));
        board.put_knight(Side::Black, coords_to_mask(6, 7));

        for file in 0..8 {
            board.put_pawn(Side::White, coords_to_mask(file, 1));
            board.put_pawn(Side::Black, coords_to_mask(file, 6));
        }

        board.update_hash();
        board.book = board.prepare_book();

        board
    }

    fn mask_to_symbol(&self, mask: u64) -> char {
        const SYMBOLS_KING: [char; 2] = ['K', 'k'];
        const SYMBOLS_QUEEN: [char; 2] = ['Q', 'q'];
        const SYMBOLS_ROOK: [char; 2] = ['R', 'r'];
        const SYMBOLS_BISHOP: [char; 2] = ['B', 'b'];
        const SYMBOLS_KNIGHT: [char; 2] = ['N', 'n'];
        const SYMBOLS_PAWN: [char; 2] = ['P', 'p'];

        let side = ((self.occupied[Side::White] & mask) == 0) as usize;

        if (self.kings[side] & mask) != 0 {
            SYMBOLS_KING[side]
        } else if (self.queens[side] & mask) != 0 {
            SYMBOLS_QUEEN[side]
        } else if (self.rooks[side] & mask) != 0 {
            SYMBOLS_ROOK[side]
        } else if (self.bishops[side] & mask) != 0 {
            SYMBOLS_BISHOP[side]
        } else if (self.knights[side] & mask) != 0 {
            SYMBOLS_KNIGHT[side]
        } else if (self.pawns[side] & mask) != 0 {
            SYMBOLS_PAWN[side]
        } else {
            '!'
        }
    }

    pub fn export_fen(&self) -> String {
        let mut result = String::new();

        for rank in 0u64..8u64 {
            let mut empty_counter = 0;

            for file in 0u64..8u64 {
                let idx = (7 - rank) * 8 + file;
                let mask = 1u64 << idx;

                if (self.any_piece & mask) == 0 {
                    empty_counter += 1;
                    continue;
                }

                if empty_counter > 0 {
                    result.push_str(format!("{}", empty_counter).as_str());
                }

                empty_counter = 0;
                result.push(self.mask_to_symbol(mask));
            }

            if empty_counter > 0 {
                result.push_str(format!("{}", empty_counter).as_str());
            }

            if rank != 7 {
                result.push('/');
            }
        }

        result.push(' ');
        result.push(match self.current_color {
            Side::White => 'w',
            Side::Black => 'b',
        });

        result.push(' ');
        if !(self.castle_kingside[Side::White]
            || self.castle_queenside[Side::White]
            || self.castle_kingside[Side::Black]
            || self.castle_queenside[Side::Black])
        {
            result.push('-');
        } else {
            if self.castle_kingside[Side::White] {
                result.push('K');
            }
            if self.castle_queenside[Side::White] {
                result.push('Q');
            }
            if self.castle_kingside[Side::Black] {
                result.push('k');
            }
            if self.castle_queenside[Side::Black] {
                result.push('q');
            }
        }

        result.push(' ');
        match self.en_passant {
            0 => result.push('-'),
            x => result.push_str(mask_to_str(x).as_str()),
        }

        result.push_str(format!(" {}", self.half_moves_clock).as_str());
        result.push_str(format!(" {}", self.full_moves_count).as_str());

        result
    }

    #[cfg(test)]
    pub fn export_graph(&self) -> String {
        let mut result = String::new();

        for rank in 0u64..8u64 {
            result.push_str(format!("{} ", 8 - rank).as_str());
            for file in 0u64..8u64 {
                let idx = (7 - rank) * 8 + file;
                let mask = 1u64 << idx;

                result.push(if (self.any_piece & mask) == 0 {
                    '.'
                } else {
                    self.mask_to_symbol(mask)
                });
                result.push(' ');
            }
            result.push('\n');
        }

        result.push_str("  A B C D E F G H ");

        result
    }

    fn put_piece_occupancy(&mut self, side: Side, mask: u64) {
        self.occupied[side] |= mask;
        self.any_piece |= mask;
    }

    fn put_king(&mut self, side: Side, mask: u64) {
        self.kings[side] |= mask;
        self.put_piece_occupancy(side, mask);
    }

    fn put_queen(&mut self, side: Side, mask: u64) {
        self.queens[side] |= mask;
        self.put_piece_occupancy(side, mask);
    }

    fn put_rook(&mut self, side: Side, mask: u64) {
        self.rooks[side] |= mask;
        self.put_piece_occupancy(side, mask);
    }

    fn put_bishop(&mut self, side: Side, mask: u64) {
        self.bishops[side] |= mask;
        self.put_piece_occupancy(side, mask);
    }

    fn put_knight(&mut self, side: Side, mask: u64) {
        self.knights[side] |= mask;
        self.put_piece_occupancy(side, mask);
    }

    fn put_pawn(&mut self, side: Side, mask: u64) {
        self.pawns[side] |= mask;
        self.put_piece_occupancy(side, mask);
    }

    fn put_piece(&mut self, side: Side, mask: u64, piece: Piece) {
        match piece {
            Piece::King => self.put_king(side, mask),
            Piece::Queen => self.put_queen(side, mask),
            Piece::Rook => self.put_rook(side, mask),
            Piece::Bishop => self.put_bishop(side, mask),
            Piece::Knight => self.put_knight(side, mask),
            Piece::Pawn => self.put_pawn(side, mask),
        }
    }

    fn remove_piece(&mut self, mask: u64) {
        self.any_piece &= !mask;
        for side in [Side::White, Side::Black] {
            self.occupied[side] &= !mask;
            self.kings[side] &= !mask;
            self.queens[side] &= !mask;
            self.rooks[side] &= !mask;
            self.bishops[side] &= !mask;
            self.knights[side] &= !mask;
            self.pawns[side] &= !mask;
        }
    }

    pub fn has_piece(&self, mask: u64) -> bool {
        self.any_piece & mask != 0
    }

    #[cfg(feature = "ui")]
    pub fn check_square(&self, mask: u64) -> Option<(Side, Piece)> {
        if !self.has_piece(mask) {
            return None;
        }

        let white_piece = self.check_piece(Side::White, mask);
        if white_piece.is_some() {
            return Some((Side::White, white_piece?));
        }

        Some((Side::Black, self.check_piece(Side::Black, mask)?))
    }

    pub fn get_attacks(&mut self, side: Side) -> u64 {
        match self.attacks[side] {
            Some(value) => value,
            None => {
                let attacks = self.generate_attacks(side);
                self.attacks[side] = Some(attacks);
                attacks
            }
        }
    }

    pub fn in_check(&mut self, side: Side) -> bool {
        match self.check[side] {
            Some(value) => value,
            None => {
                let is_in_check = self.kings[side] & self.get_attacks(!side) != 0;
                self.check[side] = Some(is_in_check);
                is_in_check
            }
        }
    }

    #[allow(dead_code)]
    pub fn in_checkmate(&mut self, side: Side) -> bool {
        match self.checkmate[side] {
            Some(value) => value,
            None => {
                let is_in_checkmate = match self.in_check(side) {
                    false => false,
                    true => self.generate_side_moves(side, false).is_empty(),
                };
                self.checkmate[side] = Some(is_in_checkmate);
                is_in_checkmate
            }
        }
    }

    pub(super) fn check_piece(&self, side: Side, mask: u64) -> Option<Piece> {
        if !self.has_piece(mask) || (self.occupied[side] & mask) == 0 {
            return None;
        }

        if (self.kings[side] & mask) != 0 {
            return Some(Piece::King);
        }

        if (self.queens[side] & mask) != 0 {
            return Some(Piece::Queen);
        }

        if (self.rooks[side] & mask) != 0 {
            return Some(Piece::Rook);
        }

        if (self.bishops[side] & mask) != 0 {
            return Some(Piece::Bishop);
        }

        if (self.knights[side] & mask) != 0 {
            return Some(Piece::Knight);
        }

        if (self.pawns[side] & mask) != 0 {
            return Some(Piece::Pawn);
        }

        panic!("Internal error: there should be something on {:066b}", mask);
    }

    pub fn key(&self) -> u64 {
        self.hash
    }

    fn check_side(&self, mask: u64) -> Side {
        if (self.occupied[Side::White] & mask) != 0 {
            return Side::White;
        }

        if (self.occupied[Side::Black] & mask) != 0 {
            return Side::Black;
        }

        eprintln!("Board history:");
        for entry in &self.history {
            eprintln!("* {}{}", idx_to_str(entry.from.trailing_zeros() as usize), idx_to_str(entry.to.trailing_zeros() as usize));
        }
        panic!("Internal error: there should be something on {} ({:#066b})", idx_to_str(mask.trailing_zeros() as usize), mask);
    }

    fn update_hash(&mut self) {
        self.hash = self.zobrist.key(self, self.castle_kingside, self.castle_queenside);
    }

    pub fn make_move(&mut self, m: Move) {
        // println!("Making move {}{}", idx_to_str(m.get_from() as usize), idx_to_str(m.get_to() as usize));
        let from_mask = 1u64 << m.get_from();
        let to_mask = 1u64 << m.get_to();

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
            self.remove_piece(to_mask);
        }

        let mut piece_type = self.check_piece(side, from_mask).unwrap();

        if piece_type == Piece::Rook {
            if self.castle_queenside[side] && from_mask == masks::ROOK_QUEENSIDE[side] {
                self.castle_queenside[side] = false;
            } else if self.castle_kingside[side] && from_mask == masks::ROOK_KINGSIDE[side] {
                self.castle_kingside[side] = false;
            }
        }

        if piece_type == Piece::King {
            if to_mask == masks::CASTLE_KINGSIDE[side] && self.castle_kingside[side] {
                self.remove_piece(masks::ROOK_KINGSIDE[side]);
                self.put_piece(side, masks::ROOK_CASTLED_KINGSIDE[side], Piece::Rook);
            } else if to_mask == masks::CASTLE_QUEENSIDE[side] && self.castle_queenside[side] {
                self.remove_piece(masks::ROOK_QUEENSIDE[side]);
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

        if piece_type == Piece::Pawn && (to_mask & masks::LAST_RANK[side]) != 0 {
            piece_type = Piece::from(m.get_promotion());
            history_entry.promotion = true;
        }

        if piece_type == Piece::Pawn && to_mask == self.en_passant {
            self.remove_piece(match side {
                Side::White => to_mask >> 8,
                Side::Black => to_mask << 8,
            });
        }

        self.en_passant = 0;

        if piece_type == Piece::Pawn
            && from_mask & masks::SECOND_RANK[side] != 0
            && to_mask & masks::EN_PASSANT_RANK[side] != 0
            && (to_mask << 1 | to_mask >> 1) & masks::EN_PASSANT_RANK[side] & self.pawns[opponent] != 0
        {
            self.en_passant = match side {
                Side::White => from_mask << 8,
                Side::Black => from_mask >> 8,
            }
        }

        self.put_piece(side, to_mask, piece_type);
        self.remove_piece(from_mask);

        self.current_color = opponent;

        if self.current_color.is_white() {
            self.full_moves_count += 1;
        }

        history_entry.attacks = self.attacks;

        self.history.push(history_entry);
        self.check = [None, None];
        self.checkmate = [None, None];
        self.attacks = [None, None];
        self.moves = [None, None];
        self.update_hash();
    }

    pub fn unmake_move(&mut self) {
        if self.history.is_empty() {
            panic!("Cannot unmake move with no moves");
        }

        let last_move = self.history.pop().unwrap();
        let side = self.check_side(last_move.to);
        let opponent = !side;

        if self.castle_kingside[side] != last_move.castle_kingside[side]
            && last_move.from == masks::KING_STARTING_POSITION[side]
            && last_move.to == masks::CASTLE_KINGSIDE[side]
            && self.check_piece(side, masks::CASTLE_KINGSIDE[side]) == Some(Piece::King)
        {
            self.remove_piece(masks::ROOK_CASTLED_KINGSIDE[side]);
            self.put_piece(side, masks::ROOK_KINGSIDE[side], Piece::Rook);
        }
        if self.castle_queenside[side] != last_move.castle_queenside[side]
            && last_move.from == masks::KING_STARTING_POSITION[side]
            && last_move.to == masks::CASTLE_QUEENSIDE[side]
            && self.check_piece(side, masks::CASTLE_QUEENSIDE[side]) == Some(Piece::King)
        {
            self.remove_piece(masks::ROOK_CASTLED_QUEENSIDE[side]);
            self.put_piece(side, masks::ROOK_QUEENSIDE[side], Piece::Rook);
        }

        self.castle_kingside = last_move.castle_kingside;
        self.castle_queenside = last_move.castle_queenside;

        let mut piece_type = self.check_piece(side, last_move.to).unwrap();

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

        self.remove_piece(last_move.to);
        self.put_piece(side, last_move.from, piece_type);

        if last_move.capture.is_some() {
            self.put_piece(opponent, last_move.to, last_move.capture.unwrap());
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

#[cfg(feature = "ui")]
impl Board {
    pub fn last_move(&self) -> Option<(u64, u64)> {
        self.history.last().map_or(None, |x| Some((x.from, x.to)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Board {
        fn make_move_str(&mut self, from: &str, to: &str) {
            self.make_move(Move::from_str(from, to));
        }

        fn assert_position(&self, fen: &str) {
            let actual = self.export_fen();
            if actual != fen {
                let actual_board = self.export_graph();
                let expected_board = Board::from_fen(fen).export_graph();
                actual_board
                    .split('\n')
                    .zip(expected_board.split('\n'))
                    .for_each(|x| println!("{:>20}  {:>20}", x.0, x.1));
            }
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
    }
}
