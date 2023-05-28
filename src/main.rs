#![allow(dead_code)]

mod layout;
mod play;
mod printers;
mod puzzle;
mod solvers;

use crate::play::play;

fn main() {
    play();
}
