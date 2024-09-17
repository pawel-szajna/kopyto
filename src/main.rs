mod board;
mod search;
mod types;
mod uci;
mod moves_generation;
mod transpositions;
mod eval;

use board::masks;

#[cfg(feature = "nn")]
fn main() {
    
}

#[cfg(not(feature = "nn"))]
fn main() {
    uci::start();
}
