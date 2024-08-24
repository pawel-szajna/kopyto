mod board;
mod ui;
mod movegen;
mod masks;

fn main() {
    let board = board::Board::from_starting_position();
    let mut ui = ui::UI::new(board);
    ui.run();
}
