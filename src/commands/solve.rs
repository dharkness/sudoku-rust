use clap::Args;
use std::collections::HashMap;
use std::io::BufRead;
use std::time::Duration;

use crate::io::{format_for_wiki, Cancelable, Parse};
use crate::puzzle::{Board, Effects, Strategy};
use crate::solve::{Difficulty, Resolution, Solver};

#[derive(Debug, Args)]
pub struct SolveArgs {
    /// Clues for a single puzzle to solve (packed format)
    puzzle: Option<String>,
}

/// Creates a new puzzle and prints it to stdout.
pub fn solve_puzzles(args: SolveArgs, cancelable: &Cancelable) {
    let solver = Solver::new();

    match args.puzzle {
        Some(puzzle) => {
            let parser = Parse::packed().stop_on_error();
            let (board, effects, failure) = parser.parse(puzzle.as_str());

            if let Some((cell, known)) = failure {
                eprintln!("error: cannot set {} to {}\n", cell, known);
            } else {
                match solver.solve(&board, &effects) {
                    Resolution::Solved(solution, difficulty, time, counts) => {
                        println!("{}", solution.packed_string('.'));
                        println!("solved {:?} in {} µs", difficulty, time.as_micros());
                        print_sorted_counts(&counts);
                    }
                    Resolution::Unsolved(last, time, counts) => {
                        println!("{}", format_for_wiki(&last));
                        println!("unsolvable in {} µs", time.as_micros());
                        print_sorted_counts(&counts);
                    }
                    Resolution::Failed(last, effects, time, counts) => {
                        println!("{}", last.packed_string('.'));
                        println!("failed in {} µs", time.as_micros());
                        print_sorted_counts(&counts);
                        effects.print_errors();
                    }
                }
            }
        }
        None => {
            let stdin = std::io::stdin();
            let parser = Parse::packed().stop_on_error();
            let mut count = 0;
            let mut solved = 0;

            for (i, puzzle) in stdin.lock().lines().enumerate() {
                count += 1;
                let (board, effects, failure) = parser.parse(&puzzle.unwrap());

                if let Some((cell, known)) = failure {
                    eprintln!("error: puzzle {} cannot set {} to {}", i + 1, cell, known);
                } else {
                    // println!("\n{}\n", board.packed_string('.'));
                    match solver.solve(&board, &effects) {
                        Resolution::Solved(_solution, difficulty, time, counts) => {
                            solved += 1;
                            log_solved(&board, difficulty, time, &counts);
                        }
                        Resolution::Unsolved(_last, time, counts) => {
                            log_unsolved(&board, time, &counts);
                        }
                        Resolution::Failed(_last, errors, time, _counts) => {
                            log_failed(&board, &errors, time);
                        }
                    }
                }

                if cancelable.is_canceled() {
                    break;
                }
            }

            eprintln!("solved {} of {} puzzles", solved, count);
        }
    }
}

fn print_sorted_counts(counts: &HashMap<Strategy, i32>) {
    let mut counts = counts.iter().collect::<Vec<(&Strategy, &i32)>>();
    counts.sort_by(|a, b| a.0.cmp(b.0));
    println!("counts: {:?}", counts);
}

fn log_solved(
    start: &Board,
    difficulty: Difficulty,
    runtime: Duration,
    counts: &HashMap<Strategy, i32>,
) {
    log_with_counts(start, format!("{:?}", difficulty).as_str(), runtime, counts);
}

fn log_unsolved(start: &Board, runtime: Duration, counts: &HashMap<Strategy, i32>) {
    log_with_counts(start, "", runtime, counts);
}

fn log_with_counts(start: &Board, text: &str, runtime: Duration, counts: &HashMap<Strategy, i32>) {
    println!(
        "{:<10} {:>8} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {}",
        text,
        runtime.as_micros(),
        // counts.get(&Strategy::Peer).unwrap_or(0),
        // counts.get(&Strategy::NakedSingle).unwrap_or(0),
        // counts.get(&Strategy::HiddenSingle).unwrap_or(0),

        counts.get(&Strategy::NakedPair).unwrap_or(&0),
        counts.get(&Strategy::NakedTriple).unwrap_or(&0),
        counts.get(&Strategy::NakedQuad).unwrap_or(&0),
        counts.get(&Strategy::HiddenPair).unwrap_or(&0),
        counts.get(&Strategy::HiddenTriple).unwrap_or(&0),
        counts.get(&Strategy::HiddenQuad).unwrap_or(&0),
        counts.get(&Strategy::PointingPair).unwrap_or(&0),
        counts.get(&Strategy::PointingTriple).unwrap_or(&0),
        counts.get(&Strategy::BoxLineReduction).unwrap_or(&0),

        counts.get(&Strategy::XWing).unwrap_or(&0),
        counts.get(&Strategy::SinglesChain).unwrap_or(&0),
        counts.get(&Strategy::YWing).unwrap_or(&0),
        counts.get(&Strategy::Swordfish).unwrap_or(&0),
        counts.get(&Strategy::XYZWing).unwrap_or(&0),

        counts.get(&Strategy::Jellyfish).unwrap_or(&0),
        counts.get(&Strategy::Skyscraper).unwrap_or(&0),
        counts.get(&Strategy::AvoidableRectangle).unwrap_or(&0),
        counts.get(&Strategy::XYChain).unwrap_or(&0),
        counts.get(&Strategy::UniqueRectangle).unwrap_or(&0),
        counts.get(&Strategy::Bug).unwrap_or(&0),

        counts.get(&Strategy::EmptyRectangle).unwrap_or(&0),

        start.packed_string('.')
    );
}

fn log_failed(start: &Board, errors: &Effects, runtime: Duration) {
    println!(
        "           {:>8}                                                                {} {:?}",
        runtime.as_micros(),
        start.packed_string('.'),
        errors.errors()
    );
}
