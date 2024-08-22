use std::str::Chars;

pub type Side = usize;
const WHITE: Side = 0;
const BLACK: Side = 1;

const MASK_ROOK_QUEENSIDE: [u64; 2] = [1u64, 1u64 << (7 * 8)];
const MASK_ROOK_KINGSIDE: [u64; 2] = [1u64 << 7, 1u64 << (7 * 8 + 7)];
const MASK_ROOK_CASTLED_QUEENSIDE: [u64; 2] = [1u64 << 3, 1u64 << (7 * 8 + 3)];
const MASK_ROOK_CASTLED_KINGSIDE: [u64; 2] = [1u64 << 5, 1u64 << (7 * 8 + 5)];
const MASK_CASTLE_QUEENSIDE: [u64; 2] = [1u64 << 2, 1u64 << (7 * 8 + 2)];
const MASK_CASTLE_KINGSIDE: [u64; 2] = [1u64 << 6, 1u64 << (7 * 8 + 6)];

#[derive(PartialEq, Eq)]
pub enum Piece {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

pub type Bitboard = u64;
pub type ColorBitboard = [Bitboard; 2];
pub type ColorBool = [bool; 2];

struct History {
    from: u64,
    to: u64,
    castle_kingside: ColorBool,
    castle_queenside: ColorBool,
    capture: Option<Piece>,
}

impl History {
    pub fn new(
        from: u64,
        to: u64,
        castle_kingside: ColorBool,
        castle_queenside: ColorBool,
    ) -> Self {
        Self {
            from,
            to,
            castle_kingside,
            castle_queenside,
            capture: None,
        }
    }
}

pub struct Board {
    kings: ColorBitboard,
    queens: ColorBitboard,
    rooks: ColorBitboard,
    bishops: ColorBitboard,
    knights: ColorBitboard,
    pawns: ColorBitboard,

    occupied: ColorBitboard,
    any_piece: Bitboard,

    castle_kingside: ColorBool,
    castle_queenside: ColorBool,

    current_color: Side,

    history: Vec<History>,

    half_moves_clock: u32,
    full_moves_count: u32,
}

fn coords_to_mask(file: usize, rank: usize) -> u64 {
    1u64 << (rank * 8usize + file)
}

fn str_to_idx(pos: &str) -> usize {
    fn get_file(pos: &mut Chars) -> usize {
        match pos.next() {
            Some('a') => 0,
            Some('b') => 1,
            Some('c') => 2,
            Some('d') => 3,
            Some('e') => 4,
            Some('f') => 5,
            Some('g') => 6,
            Some('h') => 7,
            _ => panic!("Invalid file"),
        }
    }
    fn get_rank(pos: &mut Chars) -> usize {
        match pos.next() {
            Some('1') => 0,
            Some('2') => 1,
            Some('3') => 2,
            Some('4') => 3,
            Some('5') => 4,
            Some('6') => 5,
            Some('7') => 6,
            Some('8') => 7,
            _ => panic!("Invalid rank"),
        }
    }
    let mut pos = pos.chars();
    let file = get_file(&mut pos);
    let rank = get_rank(&mut pos);
    rank * 8 + file
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

            current_color: WHITE,

            history: Vec::new(),

            half_moves_clock: 0,
            full_moves_count: 1,
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
                    Some('k') => board.put_king(BLACK, coords_to_mask(file, 7 - rank)),
                    Some('K') => board.put_king(WHITE, coords_to_mask(file, 7 - rank)),
                    Some('q') => board.put_queen(BLACK, coords_to_mask(file, 7 - rank)),
                    Some('Q') => board.put_queen(WHITE, coords_to_mask(file, 7 - rank)),
                    Some('r') => board.put_rook(BLACK, coords_to_mask(file, 7 - rank)),
                    Some('R') => board.put_rook(WHITE, coords_to_mask(file, 7 - rank)),
                    Some('p') => board.put_pawn(BLACK, coords_to_mask(file, 7 - rank)),
                    Some('P') => board.put_pawn(WHITE, coords_to_mask(file, 7 - rank)),
                    Some('n') => board.put_knight(BLACK, coords_to_mask(file, 7 - rank)),
                    Some('N') => board.put_knight(WHITE, coords_to_mask(file, 7 - rank)),
                    Some('b') => board.put_bishop(BLACK, coords_to_mask(file, 7 - rank)),
                    Some('B') => board.put_bishop(WHITE, coords_to_mask(file, 7 - rank)),
                    _ => {}
                }
                file += 1;
            }
        }

        match fen.next() {
            Some('w') => board.current_color = WHITE,
            Some('b') => board.current_color = BLACK,
            _ => panic!("Invalid fen, expected color to play"),
        }

        assert_eq!(fen.next(), Some(' '), "Invalid fen, expected space");

        board.castle_kingside = [false, false];
        board.castle_queenside = [false, false];

        loop {
            match fen.next() {
                Some('-') => { fen.next(); break },
                Some(' ') => break,
                Some('K') => board.castle_kingside[WHITE] = true,
                Some('k') => board.castle_kingside[BLACK] = true,
                Some('Q') => board.castle_queenside[WHITE] = true,
                Some('q') => board.castle_queenside[BLACK] = true,
                _ => panic!("Invalid fen, expected castling rights"),
            }
        }

        assert_eq!(fen.next(), Some('-'), "En passant in fen is not implemented yet");
        assert_eq!(fen.next(), Some(' '));

        board.half_moves_clock = 0;

        loop {
            match fen.next() {
                Some(' ') => break,
                Some(x) if x.is_digit(10) => board.half_moves_clock = board.half_moves_clock * 10 + x.to_digit(10).unwrap(),
                _ => panic!("Invalid fen, expected half move count"),
            }
        }

        board.full_moves_count = 0;

        loop {
            match fen.next() {
                Some(x) if x.is_digit(10) => board.full_moves_count = board.full_moves_count * 10 + x.to_digit(10).unwrap(),
                _ => break,
            }
        }

        board
    }

    pub fn from_starting_position() -> Board {
        let mut board = Board::new();

        board.put_king(WHITE, coords_to_mask(4, 0));
        board.put_king(BLACK, coords_to_mask(4, 7));

        board.put_queen(WHITE, coords_to_mask(3, 0));
        board.put_queen(BLACK, coords_to_mask(3, 7));

        board.put_rook(WHITE, coords_to_mask(0, 0));
        board.put_rook(WHITE, coords_to_mask(7, 0));
        board.put_rook(BLACK, coords_to_mask(0, 7));
        board.put_rook(BLACK, coords_to_mask(7, 7));

        board.put_bishop(WHITE, coords_to_mask(2, 0));
        board.put_bishop(WHITE, coords_to_mask(5, 0));
        board.put_bishop(BLACK, coords_to_mask(2, 7));
        board.put_bishop(BLACK, coords_to_mask(5, 7));

        board.put_knight(WHITE, coords_to_mask(1, 0));
        board.put_knight(WHITE, coords_to_mask(6, 0));
        board.put_knight(BLACK, coords_to_mask(1, 7));
        board.put_knight(BLACK, coords_to_mask(6, 7));

        for file in 0..8 {
            board.put_pawn(WHITE, coords_to_mask(file, 1));
            board.put_pawn(BLACK, coords_to_mask(file, 6));
        }

        board
    }

    fn mask_to_symbol(&self, mask: u64) -> char {
        const SYMBOLS_KING: [char; 2] = ['K', 'k'];
        const SYMBOLS_QUEEN: [char; 2] = ['Q', 'q'];
        const SYMBOLS_ROOK: [char; 2] = ['R', 'r'];
        const SYMBOLS_BISHOP: [char; 2] = ['B', 'b'];
        const SYMBOLS_KNIGHT: [char; 2] = ['N', 'n'];
        const SYMBOLS_PAWN: [char; 2] = ['P', 'p'];

        let side = ((self.occupied[WHITE] & mask) == 0) as usize;

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

    pub fn print_info(&self) {
        println!("occupied * {0:#066b}", self.any_piece);
        println!("occupied W {0:#066b}", self.occupied[WHITE]);
        println!("occupied B {0:#066b}", self.occupied[BLACK]);
        println!("king W     {0:#066b}", self.kings[WHITE]);
        println!("king B     {0:#066b}", self.kings[BLACK]);
        println!("queen W    {0:#066b}", self.queens[WHITE]);
        println!("queen B    {0:#066b}", self.queens[BLACK]);
        println!("bishop W   {0:#066b}", self.bishops[WHITE]);
        println!("bishop B   {0:#066b}", self.bishops[BLACK]);
        println!("knight W   {0:#066b}", self.knights[WHITE]);
        println!("knight B   {0:#066b}", self.knights[BLACK]);
        println!("pawn W     {0:#066b}", self.pawns[WHITE]);
        println!("pawn B     {0:#066b}", self.pawns[BLACK]);
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
        result.push(if self.current_color == WHITE {
            'w'
        } else {
            'b'
        });

        result.push(' ');
        if !(self.castle_kingside[WHITE]
            || self.castle_queenside[WHITE]
            || self.castle_kingside[BLACK]
            || self.castle_queenside[BLACK])
        {
            result.push('-');
        } else {
            if self.castle_kingside[WHITE] {
                result.push('K');
            }
            if self.castle_queenside[WHITE] {
                result.push('Q');
            }
            if self.castle_kingside[BLACK] {
                result.push('k');
            }
            if self.castle_queenside[BLACK] {
                result.push('q');
            }
        }

        result.push(' ');
        result.push('-'); // en passant

        result.push_str(format!(" {}", self.half_moves_clock).as_str());
        result.push_str(format!(" {}", self.full_moves_count).as_str());

        result
    }

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
        for side in WHITE..=BLACK {
            self.occupied[side] &= !mask;
            self.kings[side] &= !mask;
            self.queens[side] &= !mask;
            self.rooks[side] &= !mask;
            self.bishops[side] &= !mask;
            self.knights[side] &= !mask;
            self.pawns[side] &= !mask;
        }
    }

    fn has_piece(&self, mask: u64) -> bool {
        self.any_piece & mask != 0
    }

    fn check_piece(&self, side: Side, mask: u64) -> Option<Piece> {
        if !self.has_piece(mask) {
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

    fn check_side(&self, mask: u64) -> Side {
        if (self.occupied[WHITE] & mask) != 0 {
            return WHITE;
        }
        if (self.occupied[BLACK] & mask) != 0 {
            return BLACK;
        }
        panic!("Internal error: there should be something on {:066b}", mask);
    }

    pub fn make_move_str(&mut self, from: &str, to: &str) {
        self.make_move(str_to_idx(from), str_to_idx(to));
    }

    pub fn make_move(&mut self, from: usize, to: usize) {
        let from_mask = 1u64 << from;
        let to_mask = 1u64 << to;

        let side = self.check_side(from_mask);
        let opponent = if side == WHITE { BLACK } else { WHITE };

        let mut history_entry = History::new(
            from_mask,
            to_mask,
            self.castle_kingside,
            self.castle_queenside,
        );

        if self.has_piece(to_mask) {
            history_entry.capture = self.check_piece(opponent, to_mask);
            self.remove_piece(to_mask);
        }

        let piece_type = self.check_piece(side, from_mask).unwrap();

        if piece_type == Piece::Rook {
            if self.castle_queenside[side] && from_mask == MASK_ROOK_QUEENSIDE[side] {
                self.castle_queenside[side] = false;
            } else if self.castle_kingside[side] && from_mask == MASK_ROOK_KINGSIDE[side] {
                self.castle_kingside[side] = false;
            }
        }

        if piece_type == Piece::King {
            // TODO: check if not attacked when castling
            if self.castle_kingside[side]
                && to_mask == MASK_CASTLE_KINGSIDE[side]
                && self.check_piece(side, MASK_ROOK_KINGSIDE[side]) == Some(Piece::Rook)
            {
                self.remove_piece(MASK_ROOK_KINGSIDE[side]);
                self.put_piece(side, MASK_ROOK_CASTLED_KINGSIDE[side], Piece::Rook);
            } else if self.castle_queenside[side]
                && to_mask == MASK_CASTLE_QUEENSIDE[side]
                && self.check_piece(side, MASK_ROOK_QUEENSIDE[side]) == Some(Piece::Rook)
            {
                self.remove_piece(MASK_ROOK_QUEENSIDE[side]);
                self.put_piece(side, MASK_ROOK_CASTLED_QUEENSIDE[side], Piece::Rook);
            }
            self.castle_queenside[side] = false;
            self.castle_kingside[side] = false;
        }

        if piece_type == Piece::Pawn || history_entry.capture.is_some() {
            self.half_moves_clock = 0;
        } else {
            self.half_moves_clock += 1;
        }

        self.put_piece(side, to_mask, piece_type);
        self.remove_piece(from_mask);

        self.current_color = opponent;

        if self.current_color == WHITE {
            self.full_moves_count += 1;
        }

        self.history.push(history_entry);
    }

    pub fn unmake_move(&mut self) {
        if self.history.is_empty() {
            panic!("Cannot unmake move with no moves");
        }

        let last_move = self.history.pop().unwrap();
        let side = self.check_side(last_move.to);
        let opponent = if side == WHITE { BLACK } else { WHITE };

        if self.castle_kingside[side] != last_move.castle_kingside[side]
            && last_move.to == MASK_CASTLE_KINGSIDE[side]
        {
            self.remove_piece(MASK_ROOK_CASTLED_KINGSIDE[side]);
            self.put_piece(side, MASK_ROOK_KINGSIDE[side], Piece::Rook);
        }
        if self.castle_queenside[side] != last_move.castle_queenside[side]
            && last_move.to == MASK_CASTLE_QUEENSIDE[side]
        {
            self.remove_piece(MASK_ROOK_CASTLED_QUEENSIDE[side]);
            self.put_piece(side, MASK_ROOK_QUEENSIDE[side], Piece::Rook);
        }

        self.castle_kingside = last_move.castle_kingside;
        self.castle_queenside = last_move.castle_queenside;

        let piece_type = self.check_piece(side, last_move.to);
        self.remove_piece(last_move.to);
        self.put_piece(side, last_move.from, piece_type.unwrap());

        if last_move.capture.is_some() {
            self.put_piece(opponent, last_move.to, last_move.capture.unwrap());
        }

        self.current_color = side;
        self.half_moves_clock -= 1;
        if self.current_color == BLACK {
            self.full_moves_count -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Board {
        fn assert_position(&self, fen: &str) {
            let actual = self.export_fen();
            if actual != fen {
                let actual_board = self.export_graph();
                let expected_board = Board::from_fen(fen).export_graph();
                actual_board.split('\n').zip(expected_board.split('\n')).for_each(|x| { println!("{:>20}  {:>20}", x.0, x.1) });
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
}
