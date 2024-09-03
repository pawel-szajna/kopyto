mod chess;
mod uci;

fn main() {
    uci::UCI::new().run();
}
