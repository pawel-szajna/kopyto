mod board;
mod ui;
mod movegen;

fn main() {
    let board = board::Board::from_starting_position();
    let mut ui = ui::UI::new(board);
    ui.run();
}
