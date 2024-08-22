mod board;

fn main() {
    let board2 = board::Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    board2.print_info();

    let mut board = board::Board::from_starting_position();
    println!("{}", board.export_graph());
    println!("{}", board.export_fen());

    board.make_move_str("e2", "e4");
    println!("{}", board.export_graph());
    println!("{}", board.export_fen());

    board.unmake_move();
    println!("{}", board.export_graph());

    board.print_info();
}
