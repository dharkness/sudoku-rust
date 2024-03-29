use std::ops::RangeInclusive;
use std::time::Instant;

use clap::Args;

use crate::io::{
    format_for_wiki, format_runtime, print_all_and_single_candidates, print_known_values, Parse,
    Parser, SUDOKUWIKI_URL,
};
use crate::puzzle::{ChangeResult, Changer, Options};
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
    #[clap(short, long, default_value = "100", value_parser = max_solutions_in_range)]
    max: usize,

    /// Clues for a puzzle to solve using Bowman's Bingo
    puzzle: String,
}

/// Creates a new puzzle and prints it to stdout.
pub fn bingo(args: BingoArgs) {
    let changer = Changer::new(Options::none());
    let parser = Parse::packed_with_player(changer);

    let (mut board, effects, failure) = parser.parse(&args.puzzle);
    if !board.is_fully_solved() {
        print_all_and_single_candidates(&board);
        println!("\n=> {}{}", SUDOKUWIKI_URL, format_for_wiki(&board));
    }

    if let Some((cell, known)) = failure {
        println!("\ninvalid puzzle");
        println!("\nsetting {} to {} will cause errors\n", cell, known);
        effects.print_errors();
        return;
    }
    if let ChangeResult::Invalid(_, _, action, errors) = changer.apply_all(&board, &effects) {
        println!("\ninvalid puzzle");
        println!("\napplying {} will cause errors\n", action);
        errors.print_errors();
        return;
    }

    let runtime = Instant::now();
    let (label, empty_cells, solution, solutions) =
        match find_brute_force(&board, args.log, args.pause, args.max) {
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

    if let Some(cells) = empty_cells {
        println!(
            "\nthe puzzle has {} empty cells\n\n=> {}",
            cells.len(),
            cells
        );
    } else if let Some(solution) = solution {
        board = *solution;
    } else if let Some(solutions) = solutions {
        for (i, solution) in solutions.iter().take(10).enumerate() {
            println!("\nsolution {}\n", i + 1);
            print_known_values(solution);
            println!("\n=> {}{}", SUDOKUWIKI_URL, format_for_wiki(solution));
        }
    }

    if board.is_fully_solved() {
        println!();
        print_known_values(&board);
        println!("\n=> {}{}", SUDOKUWIKI_URL, format_for_wiki(&board));
    }
}

const MAX_SOLUTIONS_RANGE: RangeInclusive<usize> = 1..=1_000_000;

fn max_solutions_in_range(s: &str) -> Result<usize, String> {
    let max: usize = s
        .replace(',', "")
        .parse()
        .map_err(|_| format!("`{}` must be an integer", s))?;
    if MAX_SOLUTIONS_RANGE.contains(&max) {
        Ok(max)
    } else {
        Err(format!(
            "must be in range {}-{}",
            MAX_SOLUTIONS_RANGE.start(),
            MAX_SOLUTIONS_RANGE.end()
        ))
    }
}
