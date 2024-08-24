mod ui;
mod chess;

fn main() {
    let board = chess::board::Board::from_starting_position();
    let mut ui = ui::UI::new(board);
    ui.run();
}
