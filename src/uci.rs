use std::io::BufRead;
use crate::chess::board::Board;
use crate::chess::moves::Move;

pub struct UCI {
    board: Board,
}

impl UCI {
    pub fn new() -> Self {
        Self {
            board: Board::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            let mut buffer = String::new();
            std::io::stdin().read_line(&mut buffer).expect("Reading from stdin failed");
            let line = buffer.trim();
            match line {
                "quit" => break,
                "uci" => self.uci(),
                "isready" => self.isready(),
                "ucinewgame" => self.ucinewgame(),
                cmd if cmd.starts_with("position") => self.position(cmd.strip_prefix("position ")),
                _ => eprintln!("unknown command"),
            }
        }
    }

    fn uci(&self) {
        println!("id name kopyto");
        println!("id name szajnapawel@gmail.com");
        println!("uciok");
    }

    fn isready(&self) {
        println!("readyok");
    }

    fn ucinewgame(&mut self) {
        self.board = Board::new();
    }

    fn position_moves(&mut self, moves: Option<&str>) {
        match moves {
            None => {},
            Some(str) if str.trim().is_empty() => {},
            Some(moves) => {
                let space = moves.find(" ");
                let first_move = if space.is_some() { &moves[..space.unwrap()] } else { moves };
                self.board.make_move(Move::from_uci(first_move));
                let tail = moves[first_move.len()..].trim();
                self.position_moves(Some(tail));
            }
        }
    }

    fn position(&mut self, cmd: Option<&str>) {
        match cmd {
            None => eprintln!("invalid position request"),
            Some(cmd) => {
                if cmd.starts_with("startpos") {
                    self.board = Board::from_starting_position();
                    if cmd.starts_with("startpos moves ") {
                        self.position_moves(cmd.strip_prefix("startpos moves "));
                    }
                } else {
                    eprintln!("not supported");
                }
            }
        }
    }
}
