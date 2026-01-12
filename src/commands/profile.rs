use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::{Duration, Instant};

use clap::Args;

use crate::io::{Cancelable, Parse, Parser};
use crate::puzzle::{Changer, Options};
use crate::solve::{Solver, Timings};

#[derive(Debug, Args)]
pub struct ProfileArgs {
    /// File containing puzzles (one per line), or read from stdin if not provided
    file: Option<String>,

    /// Number of runs to perform
    #[clap(short, long, default_value = "1")]
    runs: usize,
}

pub fn profile_puzzles(args: ProfileArgs) {
    let cancelable = Cancelable::new();

    let puzzles: Vec<String> = match args.file {
        Some(path) => {
            let file = File::open(&path).expect("Failed to open file");
            BufReader::new(file).lines().map_while(Result::ok).collect()
        }
        None => std::io::stdin().lock().lines().map_while(Result::ok).collect(),
    };

    let count = puzzles.len();
    println!("Loaded {} puzzles, running {} time(s)", count, args.runs);

    let mut times: Vec<Duration> = Vec::with_capacity(args.runs);

    for run in 0..args.runs {
        if cancelable.is_canceled() {
            break;
        }

        let changer = Changer::new(Options::errors());
        let parser = Parse::packed_with_player(changer);
        let solver = Solver::new(false);
        let mut timings = Timings::new();
        let mut solved = 0;

        let runtime = Instant::now();

        for puzzle in &puzzles {
            if cancelable.is_canceled() {
                break;
            }
            let (board, effects, _) = parser.parse(puzzle);
            if solver.solve(&board, &effects, &mut timings).is_solved() {
                solved += 1;
            }
        }

        let elapsed = runtime.elapsed();
        times.push(elapsed);

        if args.runs > 1 {
            println!(
                "  Run {}: {:.3}ms ({}/{})",
                run + 1,
                elapsed.as_secs_f64() * 1000.0,
                solved,
                count
            );
        } else {
            println!(
                "Solved {}/{} in {:.3}ms",
                solved,
                count,
                elapsed.as_secs_f64() * 1000.0
            );
        }
    }

    if args.runs > 1 && !times.is_empty() {
        let min = times.iter().min().unwrap().as_secs_f64() * 1000.0;
        let max = times.iter().max().unwrap().as_secs_f64() * 1000.0;
        let sum: Duration = times.iter().sum();
        let avg = sum.as_secs_f64() * 1000.0 / times.len() as f64;

        println!();
        println!("Min: {:.3}ms  Max: {:.3}ms  Avg: {:.3}ms", min, max, avg);
    }
}
