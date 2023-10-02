use clap::Args;
use std::time::Instant;

use crate::io::{
    format_for_wiki, format_runtime, print_candidates, print_values, Cancelable, Parse,
    SUDOKUWIKI_URL,
};
use crate::solve::find_brute_force;

#[derive(Debug, Args)]
pub struct BingoArgs {
    /// Clues for a puzzle to solve using Bowman's Bingo
    puzzle: String,
}

/// Creates a new puzzle and prints it to stdout.
pub fn bingo(args: BingoArgs, cancelable: &Cancelable) {
    let runtime = Instant::now();
    let parser = Parse::packed().stop_on_error();
    let (board, effects, failure) = parser.parse(&args.puzzle);

    if let Some((cell, known)) = failure {
        println!(
            "failed in {} µs - {}{}\n",
            format_runtime(runtime.elapsed()),
            SUDOKUWIKI_URL,
            format_for_wiki(&board)
        );
        print_candidates(&board);
        println!("\nsetting {} to {} caused\n", cell, known);
        effects.print_errors();
        return;
    }

    if let Some(effects) = find_brute_force(&board) {
        println!("solved in {} µs\n", format_runtime(runtime.elapsed()));
        let mut clone = board.clone();
        if effects.apply_all(&mut clone).is_some() {
            println!("brute force caused errors\n");
            print_candidates(&clone);
        } else {
            print_values(&clone);
        }
    } else {
        println!("unsolvable in {} µs\n", format_runtime(runtime.elapsed()));
        print_candidates(&board);
    }
}
