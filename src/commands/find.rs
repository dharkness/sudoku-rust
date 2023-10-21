use clap::Args;
use itertools::Itertools;
use std::io::{stdin, BufRead};
use std::process::exit;
use std::time::Instant;

use crate::io::{format_number, format_runtime, print_candidates, Cancelable, Parse, Parser};
use crate::layout::CellSet;
use crate::puzzle::{Changer, Options};
use crate::solve::{Resolution, Solver};

#[derive(Debug, Args)]
pub struct FindArgs {
    /// Display the strategies used to solve each puzzle
    #[clap(short, long)]
    actions: bool,

    /// The completed puzzle to use as a starting point
    solution: String,
}

/// Applies patterns from STDIN to see if they allow a puzzle to be solved.
pub fn find_pattern(args: FindArgs, cancelable: &Cancelable) {
    let changer = Changer::new(Options::errors_and_peers());
    let parser = Parse::packed_with_player(changer);
    let (board, effects, failure) = parser.parse(&args.solution);

    if let Some((cell, known)) = failure {
        print_candidates(&board);
        eprintln!("\n==> Setting {} to {} will cause errors\n", cell, known);
        effects.print_errors();
        exit(1);
    }
    if !board.is_fully_solved() {
        print_candidates(&board);
        eprintln!("\n==> You must provide a complete solution");
        exit(1);
    }

    let runtime = Instant::now();
    let solver = Solver::new(cancelable, false);

    let mut count = 0;
    let mut solved = 0;
    let mut easiest = None;
    let mut easiest_counts = 10000;
    let mut hardest = None;
    let mut hardest_counts = 0;

    for pattern in stdin().lock().lines().map_while(Result::ok) {
        if cancelable.is_canceled() {
            break;
        }

        count += 1;
        let (start, effects) = board.with_givens(CellSet::new_from_pattern(&pattern));
        match solver.solve(&start, &effects) {
            Resolution::Canceled(..) => break,
            // Resolution::Failed(board, applied, _, action, errors) => (),
            // Resolution::Unsolved(board, applied, _) => (),
            Resolution::Solved(_, actions, difficulty) => {
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
            _ => (),
        }
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
            easiest.expect("no easiest puzzle found").packed_string(),
            easiest_counts,
            hardest.expect("no hardest puzzle found").packed_string(),
            hardest_counts,
        );
    }
}
