use clap::Args;
use std::io::BufRead;

use crate::io::{Cancelable, Parse};
use crate::puzzle::Board;
use crate::solvers::SOLVERS;

#[derive(Debug, Args)]
pub struct SolveArgs {
    /// Clues for a single puzzle to solve (packed format)
    puzzle: Option<String>,
}

/// Creates a new puzzle and prints it to stdout.
pub fn solve_puzzles(args: SolveArgs, cancelable: &Cancelable) {
    match args.puzzle {
        Some(puzzle) => {
            let parser = Parse::packed().stop_on_error();
            let (board, _, failure) = parser.parse(puzzle.as_str());

            if let Some((cell, known)) = failure {
                eprintln!("error: cannot set {} to {}\n", cell, known);
            } else if let Some(solved) = solve(&board) {
                println!("{}", solved.packed_string('.'));
            } else {
                eprintln!("error: puzzle is unsolvable");
            }
        }
        None => {
            let stdin = std::io::stdin();
            let parser = Parse::packed().stop_on_error();
            let mut count = 0;
            let mut solved = 0;

            for (i, puzzle) in stdin.lock().lines().enumerate() {
                count += 1;
                let (board, _, failure) = parser.parse(&puzzle.unwrap());

                if let Some((cell, known)) = failure {
                    eprintln!("error: puzzle {} cannot set {} to {}", i + 1, cell, known);
                } else if let Some(solution) = solve(&board) {
                    solved += 1;
                    println!("{}", solution.packed_string('.'));
                } else {
                    println!("puzzle {} cannot be solved", i + 1);
                }

                if cancelable.is_canceled() {
                    break;
                }
            }

            eprintln!("solved {} of {} puzzles", solved, count);
        }
    }
}

fn solve(board: &Board) -> Option<Board> {
    let mut clone = *board;

    loop {
        if clone.is_solved() {
            return Some(clone);
        }

        let mut found = false;
        for solver in SOLVERS {
            if let Some(effects) = solver.solve(&clone) {
                if effects.apply_all(&mut clone).is_some() {
                    return None;
                }

                found = true;
                break;
            }
        }

        if !found {
            return None;
        }
    }
}
