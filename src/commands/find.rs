use std::io::{stdin, BufRead};
use std::process::exit;
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use std::thread::{available_parallelism, spawn};
use std::time::Instant;

use clap::Args;
use itertools::Itertools;

use crate::io::{
    format_number, format_runtime, print_all_and_single_candidates, Cancelable, Parse, Parser,
};
use crate::layout::CellSet;
use crate::puzzle::{Board, Changer, Difficulty, Effects, Options};
use crate::solve::{Resolution, Solver, Timings};

#[derive(Debug, Args)]
pub struct FindArgs {
    /// Display the strategies used to solve each puzzle
    #[clap(short, long)]
    actions: bool,

    /// Worker thread count; negative values are relative to core count
    #[clap(short, long)]
    threads: Option<isize>,

    /// The completed puzzle to use as a starting point
    solution: String,
}

/// Applies patterns from STDIN and reports each one that solves the puzzle.
pub fn find_solutions(args: FindArgs) {
    let runtime = Instant::now();
    let board = parse_puzzle_or_exit(args.solution);
    let num_workers = determine_worker_count(args.threads);

    // Create channels for sending and receiving strings
    let (pattern_tx, pattern_rx) = channel();
    let (result_tx, result_rx) = channel();

    // Each worker thread will receive patterns from the shared pattern_rx channel
    let pattern_rx: Arc<Mutex<Receiver<String>>> = Arc::new(Mutex::new(pattern_rx));

    // Create worker threads
    let mut workers = Vec::with_capacity(num_workers);
    for id in 1..=num_workers {
        let pattern_rx = pattern_rx.clone();
        let result_tx = result_tx.clone();
        workers.push(spawn(move || {
            let cancelable = Cancelable::new();
            let solver = Solver::new(false);
            let runtime = Instant::now();
            let mut count = 0;
            let mut timings = Timings::new();

            loop {
                let pattern = pattern_rx.lock().unwrap().recv();
                if pattern.is_err() || cancelable.is_canceled() {
                    break;
                }
                let pattern = pattern.unwrap().to_owned();

                let (start, effects) = board.with_givens(CellSet::new_from_pattern(&pattern));
                match solver.solve(&start, &effects, &mut timings) {
                    Resolution::Canceled(..) => break,
                    Resolution::Solved(_, actions, difficulty) => {
                        result_tx
                            .send(PatternResult::Success(pattern, start, actions, difficulty))
                            .unwrap();
                    }
                    _ => {
                        result_tx
                            .send(PatternResult::Failure(pattern, start))
                            .unwrap();
                    }
                }

                count += 1;
            }

            println!(
                "{} processed {} patterns in {} µs - {} p/s",
                id,
                format_number(count),
                format_runtime(runtime.elapsed()),
                format_number((count as f64 / runtime.elapsed().as_secs_f64()) as u128)
            );
        }));
    }

    // Drop the original channel sender
    drop(result_tx);

    // Spawn a thread for reading strings from stdin
    spawn(move || {
        let cancelable = Cancelable::new();
        for line in stdin().lock().lines().map_while(Result::ok) {
            if cancelable.is_canceled() {
                break;
            }
            pattern_tx.send(line).unwrap();
        }

        // Close the channel so the workers will stop
        drop(pattern_tx);
    });

    let mut count = 0;
    let mut solved = 0;
    let mut easiest = None;
    let mut easiest_counts = 10000;
    let mut hardest = None;
    let mut hardest_counts = 0;

    // Read results from worker threads and print to stdout
    let cancelable = Cancelable::new();
    for processed in result_rx {
        if cancelable.is_canceled() {
            break;
        }

        count += 1;
        match processed {
            PatternResult::Success(_, start, actions, difficulty) => {
                solved += 1;
                println!("{} {:?}", start.packed_string(), difficulty);

                let action_count = actions.action_count();
                if action_count < easiest_counts {
                    easiest = Some(start);
                    easiest_counts = action_count;
                }
                if action_count > hardest_counts {
                    hardest = Some(start);
                    hardest_counts = action_count;
                }

                if args.actions {
                    actions
                        .action_counts()
                        .iter()
                        .sorted_by(|a, b| a.0.cmp(b.0))
                        .for_each(|(strategy, count)| {
                            println!("\n- {:>2} {:?}\n", count, strategy);
                        });
                }
            }
            PatternResult::Failure(..) => (),
        }
    }

    // Wait for all worker threads to finish
    for worker in workers {
        worker.join().unwrap();
    }

    if count > 0 {
        println!(
            "\n==> Found {} solvable puzzles from {} patterns in {} µs\n",
            format_number(solved),
            format_number(count),
            format_runtime(runtime.elapsed()),
        );
        println!(
            "    Easiest: {} - {} actions\n    Hardest: {} - {} actions",
            easiest.unwrap().packed_string(),
            easiest_counts,
            hardest.unwrap().packed_string(),
            hardest_counts,
        );
    }
}

fn determine_worker_count(requested: Option<isize>) -> usize {
    let num_cores = available_parallelism().unwrap().get() as isize;
    let count = if let Some(count) = requested {
        if count < 0 {
            num_cores + count
        } else {
            count
        }
    } else {
        num_cores - 1
    };
    if count < 1 {
        1
    } else {
        count as usize
    }
}

fn parse_puzzle_or_exit(solution: String) -> Board {
    let changer = Changer::new(Options::errors());
    let parser = Parse::packed_with_player(changer);
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

enum PatternResult {
    Success(String, Board, Effects, Difficulty),
    Failure(String, Board),
}
