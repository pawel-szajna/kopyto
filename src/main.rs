mod board;
mod search;
mod types;
mod uci;
mod moves_generation;
mod transpositions;

use board::masks;

fn main() {
    uci::start();
}
