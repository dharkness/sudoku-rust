use std::process::exit;
use std::time::Instant;

use clap::Args;
use itertools::Itertools;

use crate::build::{Finder, Generator};
use crate::io::{
    format_runtime, print_all_and_single_candidates, print_known_values, Cancelable, Parse, Parser,
};
use crate::puzzle::{Changer, Options};

#[derive(Debug, Args)]
pub struct CreateArgs {
    /// Randomize the cells before generating (can take much longer)
    #[clap(short, long)]
    randomize: bool,

    /// Stop once a puzzle with the given number of clues is found
    #[clap(short, long)]
    clues: Option<usize>,

    /// Stop after the given number of seconds
    #[clap(short, long)]
    time: Option<u64>,

    /// Show a progress bar while running
    #[clap(short, long)]
    bar: bool,

    /// The completed puzzle to use as a starting point
    #[clap(short, long)]
    solution: Option<String>,
}

/// Creates a new puzzle and prints it to stdout,
/// using the given solution and/or pattern if provided.
pub fn create_puzzle(args: CreateArgs) {
    let cancelable = Cancelable::new();
    let board = match args.solution {
        Some(solution) => {
            let parser = Parse::packed_with_options(Options::all());
            let (board, effects, failure) = parser.parse(&solution);

            if let Some((cell, known)) = failure {
                print_all_and_single_candidates(&board);
                eprintln!("\n==> Setting {} to {} will cause errors\n", cell, known);
                effects.print_errors();
                exit(1);
            }
            if !board.is_fully_solved() {
                print_all_and_single_candidates(&board);
                eprintln!("\n==> You must provide a complete solution");
                exit(1);
            }

            board
        }
        None => {
            let changer = Changer::new(Options::all());
            let mut generator = Generator::new(args.randomize, args.bar);

            match generator.generate(&changer) {
                Some(board) => {
                    if cancelable.is_canceled() {
                        print_all_and_single_candidates(&board);
                        println!("\n==> Puzzle generation canceled");
                        exit(1);
                    }
                    if !board.is_fully_solved() {
                        print_all_and_single_candidates(&board);
                        println!("\n==> Failed to generate a complete solution");
                        exit(1);
                    }

                    board
                }
                None => {
                    println!("\n==> Failed to generate a complete solution");
                    exit(1);
                }
            }
        }
    };

    print_known_values(&board);
    println!(
        "\n==> Seeking a starting puzzle for {} ...",
        board.packed_string()
    );

    let runtime = Instant::now();
    let mut finder = Finder::new(args.clues.unwrap_or(22), args.time.unwrap_or(10), args.bar);
    let (start, actions) = finder.backtracking_find(board);

    println!();
    print_all_and_single_candidates(&start);
    println!(
        "\n==> Created puzzle with {} clues in {} µs\n\n    {}\n",
        start.known_count(),
        format_runtime(runtime.elapsed()),
        start.packed_string()
    );

    let counts = actions.action_counts();
    counts
        .iter()
        .sorted_by(|a, b| a.0.cmp(b.0))
        .for_each(|(strategy, count)| {
            println!("- {:>2} {:?}", count, strategy);
        });
}
