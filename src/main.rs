#![allow(dead_code)]

use clap::{Parser, Subcommand};

use crate::commands::{
    bingo, create_puzzle, extract_patterns, find_pattern, solve_puzzles, start_player, BingoArgs,
    CreateArgs, ExtractArgs, FindArgs, PlayArgs, SolveArgs,
};
use crate::io::create_signal;

mod build;
mod commands;
mod io;
mod layout;
mod puzzle;
mod solve;
mod symbols;
mod testing;

/// A command-line sudoku player, generator and solver written in Rust
#[derive(Debug, Parser)]
#[clap(
    name = "sudoku-rust",
    version = "1.0.0",
    author = "David Harkness <dharkness@gmail.com>"
)]
#[command(propagate_version = true)]
struct App {
    #[clap(subcommand)]
    command: Option<Commands>,
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

    /// Find a solvable set of clues using patterns from STDIN
    #[clap(alias = "f")]
    Find(FindArgs),
}

/// Executes the specified subcommand.
fn main() {
    create_signal();

    let app = App::parse();
    if let Some(command) = app.command {
        match command {
            Commands::Create(args) => create_puzzle(args),
            Commands::Play(args) => start_player(args),
            Commands::Solve(args) => solve_puzzles(args),
            Commands::Bingo(args) => bingo(args),
            Commands::Extract(args) => extract_patterns(args),
            Commands::Find(args) => find_pattern(args),
        }
    } else {
        start_player(PlayArgs::new());
    }
}
