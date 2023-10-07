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
    #[clap(short = 'l', long = "log")]
    log: bool,

    /// Pause in milliseconds between each step taken
    #[clap(short = 'p', long = "pause")]
    pause: Option<u32>,

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
            println!("\nsetting {} to {} caused\n", cell, known);
        }

        effects.print_errors();
        return;
    }

    let runtime = Instant::now();
    let (label, empty_cells, solution) =
        match find_brute_force(&board, cancelable, args.log, args.pause.unwrap_or(0)) {
            BruteForceResult::AlreadySolved => ("already solved in", None, None),
            BruteForceResult::TooFewKnowns => ("not enough givens in", None, None),
            BruteForceResult::UnsolvableCells(cells) => ("unsolvable in", Some(cells), None),
            BruteForceResult::Canceled => ("canceled after", None, None),
            BruteForceResult::Unsolvable => ("unsolvable in", None, None),
            BruteForceResult::Solved(effects) => ("solved in", None, Some(effects)),
        };

    println!("\n{} {} µs", label, format_runtime(runtime.elapsed()));

    if let Some(cells) = empty_cells {
        println!(
            "\nthe puzzle has {} empty cells\n\n=> {}",
            cells.size(),
            cells
        );
    }
    if let Some(effects) = solution {
        let mut clone = board.clone();
        if let Some(errors) = effects.apply_all(&mut clone) {
            println!("\nbrute force caused errors\n");
            errors.print_errors();
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
