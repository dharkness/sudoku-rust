#![allow(dead_code)]

use clap::{Parser, Subcommand};

mod build;
mod commands;
mod io;
mod layout;
mod puzzle;
mod solve;
mod symbols;

use crate::commands::{
    bingo, create_puzzle, solve_puzzles, start_player, BingoArgs, CreateArgs, PlayArgs, SolveArgs,
};
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
    /// Generates a new complete puzzle
    #[clap(alias = "c")]
    Create(CreateArgs),

    /// Starts the interactive player
    #[clap(alias = "p")]
    Play(PlayArgs),

    /// Solves a given puzzle or all puzzles from STDIN
    #[clap(alias = "s")]
    Solve(SolveArgs),

    /// Solves a given puzzle using Bowman's Bingo
    #[clap(alias = "b")]
    Bingo(BingoArgs),
}

/// Executes the specified subcommand.
fn main() {
    let cancelable = create_signal();

    let app = App::parse();
    match app.command {
        Commands::Create(args) => create_puzzle(args, &cancelable),
        Commands::Play(args) => start_player(args, &cancelable),
        Commands::Solve(args) => solve_puzzles(args, &cancelable),
        Commands::Bingo(args) => bingo(args, &cancelable),
    }
}
