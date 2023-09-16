#![allow(dead_code)]

mod layout;
mod play;
mod printers;
mod puzzle;
mod solvers;
mod symbols;

use crate::play::play;

use rustyline::Result;

fn main() -> Result<()> {
    play()
}
