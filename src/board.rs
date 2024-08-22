pub type Side = usize;
const WHITE: Side = 0;
const BLACK: Side = 1;

pub type Bitboard = u64;
pub type ColorBitboard = [Bitboard; 2];
pub type ColorBool = [bool; 2];

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
}

fn coords_to_index(file: usize, rank: usize) -> usize {
    rank * 8 + file
}

impl Board {
    pub fn new() -> Board {
        Board {
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
        }
    }

    pub fn from_fen(fen: &str) -> Board {
        let mut board = Board::new();
        let mut fen = fen.chars();

        for rank in 0..8 {
            let mut file = 0;
            loop {
                let idx = coords_to_index(file, 7 - rank);
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
                    Some('k') => board.put_king(BLACK, idx),
                    Some('K') => board.put_king(WHITE, idx),
                    Some('q') => board.put_queen(BLACK, idx),
                    Some('Q') => board.put_queen(WHITE, idx),
                    Some('r') => board.put_rook(BLACK, idx),
                    Some('R') => board.put_rook(WHITE, idx),
                    Some('p') => board.put_pawn(BLACK, idx),
                    Some('P') => board.put_pawn(WHITE, idx),
                    Some('n') => board.put_knight(BLACK, idx),
                    Some('N') => board.put_knight(WHITE, idx),
                    Some('b') => board.put_bishop(BLACK, idx),
                    Some('B') => board.put_bishop(WHITE, idx),
                    _ => {}
                }
                file += 1;
            }
        }

        board
    }

    pub fn from_starting_position() -> Board {
        let mut board = Board::new();

        board.put_king(WHITE, coords_to_index(4, 0));
        board.put_king(BLACK, coords_to_index(4, 7));

        board.put_queen(WHITE, coords_to_index(3, 0));
        board.put_queen(BLACK, coords_to_index(3, 7));

        board.put_rook(WHITE, coords_to_index(0, 0));
        board.put_rook(WHITE, coords_to_index(7, 0));
        board.put_rook(BLACK, coords_to_index(0, 7));
        board.put_rook(BLACK, coords_to_index(7, 7));

        board.put_bishop(WHITE, coords_to_index(2, 0));
        board.put_bishop(WHITE, coords_to_index(5, 0));
        board.put_bishop(BLACK, coords_to_index(2, 7));
        board.put_bishop(BLACK, coords_to_index(5, 7));

        board.put_knight(WHITE, coords_to_index(1, 0));
        board.put_knight(WHITE, coords_to_index(6, 0));
        board.put_knight(BLACK, coords_to_index(1, 7));
        board.put_knight(BLACK, coords_to_index(6, 7));

        for file in 0..8 {
            board.put_pawn(WHITE, coords_to_index(file, 1));
            board.put_pawn(BLACK, coords_to_index(file, 6));
        }

        board
    }

    fn mask_to_symbol(&self, mask: u64) -> char {
        const SYMBOLS_KING: [char; 2] = ['K','k'];
        const SYMBOLS_QUEEN: [char; 2] = ['Q','q'];
        const SYMBOLS_ROOK: [char; 2] = ['R','r'];
        const SYMBOLS_BISHOP: [char; 2] = ['B','b'];
        const SYMBOLS_KNIGHT: [char; 2] = ['N','n'];
        const SYMBOLS_PAWN: [char; 2] = ['P','p'];

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
        result.push(if self.current_color == WHITE { 'w' } else { 'b' });

        result.push(' ');
        if self.castle_kingside[WHITE] { result.push('K'); }
        if self.castle_queenside[WHITE] { result.push('Q'); }
        if self.castle_kingside[BLACK] { result.push('k'); }
        if self.castle_queenside[BLACK] { result.push('q'); }

        result.push(' ');
        result.push('-'); // en passant

        result.push(' ');
        result.push('0'); // half-moves since last capture or pawn advance

        result.push(' ');
        result.push('1'); // full-moves since game starts

        result
    }

    pub fn export_graph(&self) -> String {
        let mut result = String::new();
        // println!("occupied * : {:#066b}", self.any_piece);
        // println!("occupied W : {:#066b}", self.occupied[WHITE]);
        // println!("occupied B : {:#066b}", self.occupied[BLACK]);
        // println!("king W     : {:#066b}", self.kings[WHITE]);
        // println!("king B     : {:#066b}", self.kings[BLACK]);
        // println!("queen W    : {:#066b}", self.queens[WHITE]);
        // println!("queen B    : {:#066b}", self.queens[BLACK]);
        for rank in 0u64..8u64 {
            result.push_str(format!("{} ", 8 - rank).as_str());
            for file in 0u64..8u64 {
                let idx = (7 - rank) * 8 + file;
                let mask = 1u64 << idx;
                // println!("rank: {}, file: {}, idx: {}, mask: {:#066b}", rank, file, idx, mask);
                if (self.any_piece & mask) == 0 {
                    result.push('.');
                } else {
                    result.push(self.mask_to_symbol(mask));
                }
                result.push(' ');
            }
            result.push('\n');
        }
        result.push_str("  A B C D E F G H");

        result
    }

    fn put_piece(&mut self, side: Side, mask: u64) {
        self.occupied[side] |= mask;
        self.any_piece |= mask;
    }

    fn put_king(&mut self, side: Side, square: usize) {
        let mask = 1u64 << square;
        self.kings[side] |= mask;
        self.put_piece(side, mask);
    }

    fn put_queen(&mut self, side: Side, square: usize) {
        let mask = 1u64 << square;
        self.queens[side] |= mask;
        self.put_piece(side, mask);
    }

    fn put_rook(&mut self, side: Side, square: usize) {
        let mask = 1u64 << square;
        self.rooks[side] |= mask;
        self.put_piece(side, mask);
    }

    fn put_bishop(&mut self, side: Side, square: usize) {
        let mask = 1u64 << square;
        self.bishops[side] |= mask;
        self.put_piece(side, mask);
    }

    fn put_knight(&mut self, side: Side, square: usize) {
        let mask = 1u64 << square;
        self.knights[side] |= mask;
        self.put_piece(side, mask);
    }

    fn put_pawn(&mut self, side: Side, square: usize) {
        let mask = 1u64 << square;
        self.pawns[side] |= mask;
        self.put_piece(side, mask);
    }

    // fn remove_piece(mut self, side: Side, square: u8) {
    //     self.any_piece &= !(1 << square);
    // }
}
