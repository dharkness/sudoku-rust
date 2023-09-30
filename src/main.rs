#![allow(dead_code)]

use clap::{Parser, Subcommand};

mod build;
mod commands;
mod io;
mod layout;
mod puzzle;
mod solvers;
mod symbols;

use crate::commands::{create_puzzle, start_player, CreateArgs, PlayArgs};
use crate::io::create_signal;

/// A command-line sudoku player, generator and solver written in Rust
#[derive(Debug, Parser)]
#[clap(
    name = "sudoku-rust",
    version = "0.2.0",
    author = "David Harkness <dharkness@gmail.com>"
)]
pub struct App {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Starts the interactive player
    #[clap(alias = "p")]
    Play(PlayArgs),
    /// Generates a new complete puzzle
    #[clap(alias = "c")]
    Create(CreateArgs),
}

/// Starts the interactive player or creates a new puzzle.
fn main() {
    let canceler = create_signal();

    let app = App::parse();
    match app.command {
        Commands::Play(args) => start_player(args, &canceler),
        Commands::Create(args) => create_puzzle(args, &canceler),
    }
}
