use crate::chess::board::Board;
use crate::chess::moves::Move;
use crate::chess::search;
use crate::chess::search::Search;
use std::io::BufRead;

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
            std::io::stdin()
                .read_line(&mut buffer)
                .expect("Reading from stdin failed");
            let line = buffer.trim();
            match line {
                "quit" => break,
                "uci" => self.uci(),
                "isready" => self.isready(),
                "ucinewgame" => self.ucinewgame(),
                cmd if cmd.starts_with("position") => self.position(cmd.strip_prefix("position ")),
                cmd if cmd.starts_with("go") => self.go(cmd.strip_prefix("go ")),
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
            None => {}
            Some(str) if str.trim().is_empty() => {}
            Some(moves) => {
                let space = moves.find(" ");
                let first_move = if space.is_some() {
                    &moves[..space.unwrap()]
                } else {
                    moves
                };
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

    fn parse_go_options(&self, options: &mut search::Options, cmd: Option<&str>) {
    }

    fn go(&mut self, cmd: Option<&str>) {
        let mut options = search::Options::new();
        self.parse_go_options(&mut options, cmd);

        let result = self.board.search();
        eprintln!("Found move {:?}, eval {}", result.m, result.score);
        println!("bestmove {}", result.m.to_uci());
    }
}
