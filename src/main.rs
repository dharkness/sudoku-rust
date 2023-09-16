#![allow(dead_code)]

mod io;
mod layout;
mod play;
mod puzzle;
mod solvers;
mod symbols;

use crate::play::play;

fn main() {
    play();
}
