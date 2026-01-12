use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Instant;

use clap::Args;

use crate::io::{format_number, format_runtime, Cancelable, Parse, Parser};
use crate::puzzle::{Changer, Options};
use crate::solve::{Solver, Timings};

#[derive(Debug, Args)]
pub struct ProfileArgs {
    /// File containing puzzles (one per line), or read from stdin if not provided
    file: Option<String>,
}

pub fn profile_puzzles(args: ProfileArgs) {
    let cancelable = Cancelable::new();
    let changer = Changer::new(Options::errors());
    let parser = Parse::packed_with_player(changer);
    let solver = Solver::new(false);
    let mut timings = Timings::new();

    let puzzles: Vec<String> = match args.file {
        Some(path) => {
            let file = File::open(&path).expect("Failed to open file");
            BufReader::new(file).lines().map_while(Result::ok).collect()
        }
        None => std::io::stdin().lock().lines().map_while(Result::ok).collect(),
    };

    let count = puzzles.len();
    println!("Loaded {} puzzles", format_number(count as u128));

    let runtime = Instant::now();
    let mut solved = 0;

    for puzzle in puzzles {
        if cancelable.is_canceled() {
            break;
        }
        let (board, effects, _) = parser.parse(&puzzle);
        if solver.solve(&board, &effects, &mut timings).is_solved() {
            solved += 1;
        }
    }

    let elapsed = runtime.elapsed();
    println!(
        "Solved {} of {} puzzles in {} µs",
        format_number(solved as u128),
        format_number(count as u128),
        format_runtime(elapsed)
    );
}
