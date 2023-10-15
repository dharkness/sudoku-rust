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
    bingo, create_puzzle, extract_patterns, solve_puzzles, start_player, BingoArgs, CreateArgs,
    ExtractArgs, PlayArgs, SolveArgs,
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
    /// Generate a new complete puzzle
    #[clap(alias = "c")]
    Create(CreateArgs),

    /// Start the interactive player
    #[clap(alias = "p")]
    Play(PlayArgs),

    /// Solve a puzzle or all puzzles from STDIN
    #[clap(alias = "s")]
    Solve(SolveArgs),

    /// Brute force a puzzle using Bowman's Bingo
    #[clap(alias = "b")]
    Bingo(BingoArgs),

    /// Extract patterns from puzzles from STDIN
    #[clap(alias = "e")]
    Extract(ExtractArgs),
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
        Commands::Extract(args) => extract_patterns(args, &cancelable),
    }
}
