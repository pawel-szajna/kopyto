mod board;
mod eval;
mod moves_generation;
mod search;
mod transpositions;
mod types;
mod uci;

use board::masks;

fn main() {
    uci::start();
}
