mod board;
mod book;
mod search;
mod types;
mod uci;
mod util;
mod moves_generation;
mod transpositions;

use board::masks;

fn main() {
    uci::start();
}
