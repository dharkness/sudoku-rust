mod effects;
mod generate;
mod layout;
mod play;
mod printers;
mod solvers;

use crate::generate::generate_board;
use crate::play::play_puzzle;

fn main() {
    // play_puzzle();
    generate_board();
}
