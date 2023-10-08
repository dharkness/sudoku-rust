use clap::Args;
use std::time::Instant;

use crate::io::{
    format_for_wiki, format_runtime, print_candidates, print_values, Cancelable, Parse,
    SUDOKUWIKI_URL,
};
use crate::solve::{find_brute_force, BruteForceResult};

#[derive(Debug, Args)]
pub struct BingoArgs {
    /// Log each cell and candidate tried
    #[clap(short, long)]
    log: bool,

    /// Pause in milliseconds between each step taken
    #[clap(short, long, default_value = "0")]
    pause: u32,

    /// Maximum number of solutions to find before stopping
    #[clap(short, long, default_value = "10")]
    max: usize,

    /// Clues for a puzzle to solve using Bowman's Bingo
    puzzle: String,
}

/// Creates a new puzzle and prints it to stdout.
pub fn bingo(args: BingoArgs, cancelable: &Cancelable) {
    let parser = Parse::packed().stop_on_error().remove_peers();
    let (mut board, effects, failure) = parser.parse(&args.puzzle);

    if !board.is_solved() {
        print_candidates(&board);
        println!("\n=> {}{}", SUDOKUWIKI_URL, format_for_wiki(&board));
    }

    effects.apply_all(&mut board);
    if effects.has_errors() {
        println!("\ninvalid puzzle");
        if let Some((cell, known)) = failure {
            println!("\nsetting {} to {} will cause errors\n", cell, known);
        }

        effects.print_errors();
        return;
    }

    let runtime = Instant::now();
    let (label, empty_cells, solution, solutions) =
        match find_brute_force(&board, cancelable, args.log, args.pause, args.max) {
            BruteForceResult::AlreadySolved => ("already solved in".to_string(), None, None, None),
            BruteForceResult::TooFewKnowns => {
                ("not enough givens in".to_string(), None, None, None)
            }
            BruteForceResult::UnsolvableCells(cells) => {
                ("unsolvable in".to_string(), Some(cells), None, None)
            }
            BruteForceResult::Canceled => ("canceled after".to_string(), None, None, None),
            BruteForceResult::Unsolvable => ("unsolvable in".to_string(), None, None, None),
            BruteForceResult::Solved(solution) => {
                ("solved in".to_string(), None, Some(solution), None)
            }
            BruteForceResult::MultipleSolutions(solutions) => (
                format!("found {} solutions in", solutions.len()),
                None,
                None,
                Some(solutions),
            ),
        };

    println!("\n{} {} µs", label, format_runtime(runtime.elapsed()));

    if let Some(solutions) = solutions {
        for (i, solution) in solutions.iter().enumerate() {
            if i == 10 {
                break;
            }
            let mut clone = board;
            solution.apply_all(&mut clone);
            println!("\nsolution {}\n", i + 1);
            print_values(&clone);
            println!("\n=> {}{}", SUDOKUWIKI_URL, format_for_wiki(&clone));
        }
    }
    if let Some(cells) = empty_cells {
        println!(
            "\nthe puzzle has {} empty cells\n\n=> {}",
            cells.size(),
            cells
        );
    }
    if let Some(solution) = solution {
        let mut clone = board;
        if let Some(errors) = solution.apply_all(&mut clone) {
            println!("\nbrute force caused errors\n");
            errors.print_errors();
            println!();
            print_candidates(&clone);
        } else {
            board = clone;
        }
    }

    if board.is_solved() {
        println!();
        print_values(&board);
        println!("\n=> {}{}", SUDOKUWIKI_URL, format_for_wiki(&board));
    }
}
