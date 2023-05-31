#![allow(dead_code)]

extern crate core;

mod layout;
mod play;
mod printers;
mod puzzle;
mod solvers;
mod symbols;

use crate::play::play;

fn main() {
    play();
}
