mod board;
mod book;
mod magics;
mod masks;
mod moves_generation;
mod search;
mod transpositions;
mod types;
mod uci;
mod util;

fn main() {
    uci::start();
}
