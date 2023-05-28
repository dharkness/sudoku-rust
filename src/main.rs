#![allow(dead_code)]

mod effects;
mod generate;
mod layout;
mod play;
mod printers;
mod solvers;

use crate::play::play;

fn main() {
    play();
}
