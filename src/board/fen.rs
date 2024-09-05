use crate::board::Board;
use crate::types::{Bitboard, Side, Square};

pub trait FenConsumer {
    fn from_fen(fen: &str) -> Self;
}

pub trait FenProducer {
    fn export_fen(&self) -> String;
}

impl FenConsumer for Board {
    fn from_fen(fen: &str) -> Self {
        let mut board = Self::new();
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
                    Some('k') => board.put_king(Side::Black, Bitboard::from_coords(file, 7 - rank)),
                    Some('K') => board.put_king(Side::White, Bitboard::from_coords(file, 7 - rank)),
                    Some('q') => board.put_queen(Side::Black, Bitboard::from_coords(file, 7 - rank)),
                    Some('Q') => board.put_queen(Side::White, Bitboard::from_coords(file, 7 - rank)),
                    Some('r') => board.put_rook(Side::Black, Bitboard::from_coords(file, 7 - rank)),
                    Some('R') => board.put_rook(Side::White, Bitboard::from_coords(file, 7 - rank)),
                    Some('p') => board.put_pawn(Side::Black, Bitboard::from_coords(file, 7 - rank)),
                    Some('P') => board.put_pawn(Side::White, Bitboard::from_coords(file, 7 - rank)),
                    Some('n') => board.put_knight(Side::Black, Bitboard::from_coords(file, 7 - rank)),
                    Some('N') => board.put_knight(Side::White, Bitboard::from_coords(file, 7 - rank)),
                    Some('b') => board.put_bishop(Side::Black, Bitboard::from_coords(file, 7 - rank)),
                    Some('B') => board.put_bishop(Side::White, Bitboard::from_coords(file, 7 - rank)),
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
                    // TODO: fix this monstrosity
                    board.en_passant = Bitboard::from(Square::from(format!("{}{}", file, rank).as_str()))
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
}

fn mask_to_symbol(board: &Board, mask: Bitboard) -> char {
    const SYMBOLS_KING: [char; 2] = ['K', 'k'];
    const SYMBOLS_QUEEN: [char; 2] = ['Q', 'q'];
    const SYMBOLS_ROOK: [char; 2] = ['R', 'r'];
    const SYMBOLS_BISHOP: [char; 2] = ['B', 'b'];
    const SYMBOLS_KNIGHT: [char; 2] = ['N', 'n'];
    const SYMBOLS_PAWN: [char; 2] = ['P', 'p'];

    let side = (board.occupied[Side::White] & mask).empty() as usize;

    if !(board.kings[side] & mask).empty() {
        SYMBOLS_KING[side]
    } else if !(board.queens[side] & mask).empty() {
        SYMBOLS_QUEEN[side]
    } else if !(board.rooks[side] & mask).empty() {
        SYMBOLS_ROOK[side]
    } else if !(board.bishops[side] & mask).empty() {
        SYMBOLS_BISHOP[side]
    } else if !(board.knights[side] & mask).empty() {
        SYMBOLS_KNIGHT[side]
    } else if !(board.pawns[side] & mask).empty() {
        SYMBOLS_PAWN[side]
    } else {
        '!'
    }
}

impl FenProducer for Board {
    fn export_fen(&self) -> String {
        let mut result = String::new();

        for rank in 0..8 {
            let mut empty_counter = 0;

            for file in 0..8 {
                let idx = Square::from((7 - rank) * 8 + file);
                let mask = Bitboard::from(idx);

                if (self.any_piece & mask).empty() {
                    empty_counter += 1;
                    continue;
                }

                if empty_counter > 0 {
                    result.push_str(format!("{}", empty_counter).as_str());
                }

                empty_counter = 0;
                result.push(mask_to_symbol(self, mask));
            }

            if empty_counter > 0 {
                result.push_str(format!("{}", empty_counter).as_str());
            }

            if rank != 7 {
                result.push('/');
            }
        }

        result.push(' ');
        result.push(match self.side_to_move() {
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
            Bitboard::EMPTY => result.push('-'),
            x => result.push_str(x.peek().to_string().as_str()),
        }

        result.push_str(format!(" {}", self.half_moves_clock).as_str());
        result.push_str(format!(" {}", self.full_moves_count).as_str());

        result
    }
}