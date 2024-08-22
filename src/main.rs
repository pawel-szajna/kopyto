mod board;

fn main() {
    let board = board::Board::from_starting_position();
    println!("{}", board.export_graph());
    println!("{}", board.export_fen());

    let board2 = board::Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    println!("{}", board2.export_graph());
    println!("{}", board2.export_fen());
}
