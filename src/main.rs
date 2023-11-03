#![allow(dead_code)]

use clap::{Parser, Subcommand};

use crate::commands::{
    bingo, create_puzzle, extract_patterns, find_solutions, solve_puzzles, start_player, BingoArgs,
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
    /// Start the interactive player
    ///
    /// You may set some auto-solving options and pass a starting puzzle.
    /// All of the commands are displayed as a help menu upon startup.
    #[clap(alias = "p", verbatim_doc_comment)]
    Play(PlayArgs),

    /// Generate a new complete puzzle and starting clues
    ///
    /// If you do not provide a completed puzzle, this starts by creating one.
    /// You may specify the target number of clues, the maximum time to look
    /// for a minimal starting puzzle, and a progress bar.
    #[clap(alias = "c", verbatim_doc_comment)]
    Create(CreateArgs),

    /// Solve given puzzles or all puzzles from STDIN
    ///
    /// If you provide starting clues on the command line, each will be solved
    /// and its solution printed with details about the strategies employed.
    ///
    /// If not, puzzles will be read from STDIN, one per line, and solved.
    /// The solution will be printed with the number of times each strategy
    /// is employed in a tabular format.
    ///
    /// If the puzzle cannot be solved or becomes invalid while solving it,
    /// you can use the `--check` option to validate the puzzle after each
    /// strategy to find which strategy may be faulty.
    #[clap(alias = "s", verbatim_doc_comment)]
    Solve(SolveArgs),

    /// Brute force a puzzle using Bowman's Bingo
    ///
    /// Finds all possible solutions for a starting puzzle, up to a maximum.
    /// Use `--max` to change the maximum number of solutions to find.
    ///
    /// If you want to see how it works, you can set the `--log` option
    /// and a `--pause` between each step.
    #[clap(alias = "b", verbatim_doc_comment)]
    Bingo(BingoArgs),

    /// Extract patterns from puzzles from STDIN
    ///
    /// This is useful for finding patterns that you can use when creating
    /// new puzzles with the `find` command. Redirect a file of puzzles to it,
    /// and it will print each pattern on its own line when redirected to STDOUT.
    ///
    /// When not redirecting to STDOUT, it will count the number of times each
    /// pattern is found and print them sorted by the counts along with some
    /// statistics about number of clues. Use the `--total` option to skip
    /// the list of patterns and only print the totals.
    #[clap(alias = "e", verbatim_doc_comment)]
    Extract(ExtractArgs),

    /// Find a solvable set of clues using patterns from STDIN
    ///
    /// Redirect a file containing patterns to this command, and it will print each
    /// one that provides a unique solution to the given puzzle. By default it will
    /// use all available CPU cores, but you can specify a different number with
    /// the `--threads` option.
    ///
    /// Add the `--actions` option to print the strategies employed to solve each puzzle.
    #[clap(alias = "f", verbatim_doc_comment)]
    Find(FindArgs),
}

/// Executes the specified subcommand.
fn main() {
    create_signal();

    let app = App::parse();
    if let Some(command) = app.command {
        match command {
            Commands::Play(args) => start_player(args),
            Commands::Create(args) => create_puzzle(args),
            Commands::Solve(args) => solve_puzzles(args),
            Commands::Bingo(args) => bingo(args),
            Commands::Extract(args) => extract_patterns(args),
            Commands::Find(args) => find_solutions(args),
        }
    } else {
        start_player(PlayArgs::new());
    }
}
