mod board;
mod book;
mod search;
mod types;
mod uci;
mod util;
mod moves_generation;
mod transpositions;

use board::masks;
use board::magics;

fn main() {
    uci::start();
}
