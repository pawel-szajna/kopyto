use std::str::FromStr;
use scanner_rust::ScannerAscii;
use crate::board::{Board, FenConsumer, FenProducer};
use crate::types::Move;
use crate::moves_generation::perft;
use crate::search;
use crate::search::{Searcher, Verbosity};
use crate::transpositions::Transpositions;

pub struct UCI {
    board: Board,
    last_position: String,
    book: bool,
    transpositions: Transpositions,
}

impl UCI {
    pub fn new() -> Self {
        Self {
            board: Board::from_starting_position(),
            last_position: String::new(),
            book: false,
            transpositions: Transpositions::new(64),
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
                "stop" => (),
                "uci" => self.uci(),
                "eval" => self.eval(),
                "isready" => self.isready(),
                "ucinewgame" => self.ucinewgame(),
                "currentfen" => println!("{}", self.board.export_fen()),
                cmd if cmd.starts_with("position") => self.position(cmd.strip_prefix("position ")),
                cmd if cmd.starts_with("go") => self.go(cmd.strip_prefix("go").unwrap_or("").trim()),
                cmd if cmd.starts_with("setoption") => self.setoption(cmd.strip_prefix("setoption").unwrap_or("").trim()),
                _ => println!("info string unknown command"),
            }
        }
    }

    fn uci(&self) {
        println!("id name kopyto");
        println!("id author szajnapawel@gmail.com");
        println!("option name Book type check default false");
        println!("option name Hash type spin default 64 min 1 max 2048");
        println!("uciok");
    }

    fn isready(&self) {
        println!("readyok");
    }

    fn ucinewgame(&mut self) {
        self.board = Board::new();
    }

    fn setoption(&mut self, option: &str) {
        let mut scanner = ScannerAscii::new(option.as_bytes());
        let mut option_name = String::new();
        let mut option_value = String::new();

        loop {
            match scanner.next() {
                Err(e) => println!("info string cannot parse option: {}", e),
                Ok(mode) => match mode {
                    None => break,
                    Some(mode) => match mode.as_str() {
                        "name" => option_name = scanner.next().unwrap().unwrap(),
                        "value" => option_value = scanner.next().unwrap().unwrap(),
                        _ => (),
                    }
                }
            }
        }

        match option_name.as_str() {
            "Book" => self.book = bool::from_str(option_value.as_str()).unwrap(),
            "Hash" => self.transpositions = Transpositions::new(usize::from_str(option_value.as_str()).unwrap()),
            _ => println!("unknown option: {}, ignoring", option_name),
        }
    }

    fn position_moves(&mut self, moves: Option<&str>) {
        match moves {
            None => {}
            Some(str) if str.trim().is_empty() => {}
            Some(str) if str.starts_with(" ") => self.position_moves(Some(str.trim())),
            Some(str) if str.trim().starts_with("moves ") => self.position_moves(str.trim().strip_prefix("moves ")),
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
            None => println!("info string invalid position request"),
            Some(cmd) => match cmd {
                cmd if cmd.starts_with("startpos") => {
                    if !self.last_position.is_empty() && cmd.starts_with(self.last_position.as_str()) {
                        let remainder = cmd.strip_prefix(self.last_position.as_str());
                        self.position_moves(remainder);
                    } else {
                        self.board = Board::from_starting_position();
                        if cmd.starts_with("startpos moves ") {
                            self.position_moves(cmd.strip_prefix("startpos moves "));
                        }
                    }
                    self.last_position = String::from(cmd);
                }
                cmd if cmd.starts_with("fen ") => {
                    let moves = cmd.find(" moves ");
                    let fen = if moves.is_some() {
                        &cmd[4..moves.unwrap()]
                    } else {
                        &cmd[4..]
                    };
                    self.board = Board::from_fen(fen);
                    if moves.is_some() {
                        let moves_str = &cmd[(moves.unwrap() + 7)..];
                        self.position_moves(Some(moves_str));
                    }
                }
                _ => println!("info string unknown position format"),
            },
        }
    }

    fn parse_go_options(&self, options: &mut search::Options, cmd: &str) {
        let mut scanner = ScannerAscii::new(cmd.as_bytes());
        loop {
            match scanner.next() {
                Err(e) => println!("info string parsing failed: {}", e),
                Ok(result) => match result {
                    None => break,
                    Some(command) => match command.as_str() {
                        "infinite" => options.depth = None,
                        "depth" => options.depth = scanner.next_i16().unwrap(),
                        "wtime" => options.white_time = scanner.next_i32().unwrap().unwrap(),
                        "btime" => options.black_time = scanner.next_i32().unwrap().unwrap(),
                        "winc" => options.white_increment = scanner.next_i32().unwrap().unwrap(),
                        "binc" => options.black_increment = scanner.next_i32().unwrap().unwrap(),
                        "movetime" => options.target_time = scanner.next_i32().unwrap(),
                        _ => (),
                    }
                },
            }
        }
    }

    fn go(&mut self, cmd: &str) {
        if cmd.starts_with("perft") {
            let cmd = cmd.strip_prefix("perft ");
            let depth = cmd.unwrap().to_string().parse::<usize>();
            match depth {
                Err(what) => panic!("cannot parse: {}", what.to_string()),
                Ok(depth) => { perft(&mut self.board, depth); },
            }
            return;
        }

        let mut options = search::Options::new();
        self.parse_go_options(&mut options, cmd);
        let mut searcher = Searcher::new(self.board.clone(), &mut self.transpositions, self.book);
        let result = searcher.go(options);

        println!("bestmove {}", result.to_uci());
    }

    fn eval(&self) {
        println!("{}", search::evaluate(&self.board, Verbosity::Verbose));
    }
}
