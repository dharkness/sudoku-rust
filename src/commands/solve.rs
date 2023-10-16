use clap::Args;
use itertools::Itertools;
use std::collections::HashMap;
use std::io::BufRead;
use std::time::{Duration, Instant};

use crate::io::{
    format_for_wiki, format_number, format_runtime, print_candidates, print_values, Cancelable,
    ParsePacked, SUDOKUWIKI_URL,
};
use crate::layout::{Cell, Known};
use crate::puzzle::{Action, Board, Effects, Options, Player, Strategy};
use crate::solve::{Difficulty, Reporter, Resolution, Solver};

#[derive(Debug, Args)]
pub struct SolveArgs {
    /// Check the results of each solver strategy using brute force
    #[clap(short, long)]
    check: bool,

    /// Clues for one or more puzzles to solve with detailed output
    puzzles: Option<Vec<String>>,
}

/// Creates a new puzzle and prints it to stdout.
pub fn solve_puzzles(args: SolveArgs, cancelable: &Cancelable) {
    let player = Player::new(Options::errors_and_peers());
    let parser = ParsePacked::new_with_player(player);
    let solver = Solver::new(cancelable, args.check);

    match args.puzzles {
        Some(puzzles) => {
            let reporter = DetailedReporter::new();
            let parser_solver = ParserSolver::new(&parser, &solver, &reporter);

            for puzzle in puzzles {
                parser_solver.parse_and_solve(&puzzle);
                if cancelable.is_canceled() {
                    break;
                }
            }
        }
        None => {
            let reporter = CSVReporter::new();
            let parser_solver = ParserSolver::new(&parser, &solver, &reporter);
            let stdin = std::io::stdin();

            let runtime = Instant::now();
            let mut count = 0;
            let mut solved = 0;

            println!("                   µs NS HS NP NT NQ HP HT HQ PP PT BL XW SC YW SF XZ JF SK AR XY UR BG ER");
            for puzzle in stdin.lock().lines().map_while(Result::ok) {
                if cancelable.is_canceled() {
                    break;
                }
                if parser_solver.parse_and_solve(&puzzle) {
                    solved += 1;
                }
                count += 1;
            }

            println!(
                "solved {} of {} puzzles in {} µs",
                format_number(solved),
                format_number(count),
                format_runtime(runtime.elapsed())
            );
        }
    }
}

struct ParserSolver<'a> {
    parser: &'a ParsePacked,
    solver: &'a Solver<'a>,
    reporter: &'a dyn Reporter,
}

impl ParserSolver<'_> {
    fn new<'a>(
        parser: &'a ParsePacked,
        solver: &'a Solver<'a>,
        reporter: &'a dyn Reporter,
    ) -> ParserSolver<'a> {
        ParserSolver {
            parser,
            solver,
            reporter,
        }
    }

    fn parse_and_solve(&self, givens: &str) -> bool {
        let runtime = Instant::now();
        let (start, effects, failure) = self.parser.parse(givens);

        if let Some((cell, known)) = failure {
            self.reporter
                .invalid(givens, &start, &effects, cell, known, runtime.elapsed());
            return false;
        }

        match self.solver.solve(&start, &effects) {
            Resolution::Canceled(..) => (),
            Resolution::Failed(board, applied, _, action, errors) => self.reporter.failed(
                givens,
                &start,
                &board,
                &action,
                &errors,
                runtime.elapsed(),
                &applied.action_counts(),
            ),
            Resolution::Unsolved(board, applied, _) => self.reporter.unsolved(
                givens,
                &start,
                &board,
                runtime.elapsed(),
                &applied.action_counts(),
            ),
            Resolution::Solved(solution, actions, difficulty) => {
                self.reporter.solved(
                    givens,
                    &start,
                    &solution,
                    difficulty,
                    runtime.elapsed(),
                    &actions.action_counts(),
                );
                return true;
            }
        }

        false
    }
}

struct DetailedReporter {}

impl DetailedReporter {
    fn new() -> DetailedReporter {
        DetailedReporter {}
    }

    fn print_counts(&self, counts: &HashMap<Strategy, i32>) {
        counts
            .iter()
            .sorted_by(|a, b| a.0.cmp(b.0))
            .for_each(|(strategy, count)| {
                println!("- {:>2} {:?}", count, strategy);
            });
    }
}

impl Reporter for DetailedReporter {
    fn invalid(
        &self,
        _givens: &str,
        partial: &Board,
        errors: &Effects,
        cell: Cell,
        known: Known,
        runtime: Duration,
    ) {
        println!("invalid in {} µs\n", format_runtime(runtime));
        print_candidates(partial);
        println!("\nsetting {} to {} will cause errors\n", cell, known);
        errors.print_errors();
    }

    fn failed(
        &self,
        _givens: &str,
        _start: &Board,
        stopped: &Board,
        action: &Action,
        errors: &Effects,
        runtime: Duration,
        counts: &HashMap<Strategy, i32>,
    ) {
        println!(
            "failed in {} µs - {}{}\n",
            format_runtime(runtime),
            SUDOKUWIKI_URL,
            format_for_wiki(stopped)
        );
        print_candidates(stopped);
        println!("\ncaused by {:?} - {}\n", action.strategy(), action);
        errors.print_errors();
        println!();
        self.print_counts(counts);
    }

    fn unsolved(
        &self,
        _givens: &str,
        _start: &Board,
        stopped: &Board,
        runtime: Duration,
        counts: &HashMap<Strategy, i32>,
    ) {
        println!("unsolved in {} µs\n", format_runtime(runtime));
        println!(
            "stopped at {}{}\n",
            SUDOKUWIKI_URL,
            format_for_wiki(stopped)
        );
        print_candidates(stopped);
        println!();
        self.print_counts(counts);
    }

    fn solved(
        &self,
        _givens: &str,
        _start: &Board,
        solution: &Board,
        difficulty: Difficulty,
        runtime: Duration,
        counts: &HashMap<Strategy, i32>,
    ) {
        println!(
            "solved {:?} in {} µs - {}\n",
            difficulty,
            format_runtime(runtime),
            solution.packed_string()
        );
        print_values(solution);
        println!();
        self.print_counts(counts);
    }
}

struct CSVReporter {}

impl CSVReporter {
    fn new() -> CSVReporter {
        CSVReporter {}
    }

    fn format_counts(&self, counts: &HashMap<Strategy, i32>) -> String {
        format!(
            "{:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2}",
            // counts.get(&Strategy::Peer).unwrap_or(0),
            // counts.get(&Strategy::NakedSingle).unwrap_or(0),
            // counts.get(&Strategy::HiddenSingle).unwrap_or(0),

            counts.get(&Strategy::NakedSingle).unwrap_or(&0),
            counts.get(&Strategy::HiddenSingle).unwrap_or(&0),

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
        )
    }
}

impl Reporter for CSVReporter {
    fn invalid(
        &self,
        givens: &str,
        _partial: &Board,
        _errors: &Effects,
        cell: Cell,
        known: Known,
        _runtime: Duration,
    ) {
        eprintln!("invalid: cannot set {} to {} for {}", cell, known, givens);
    }

    fn failed(
        &self,
        _givens: &str,
        start: &Board,
        _stopped: &Board,
        action: &Action,
        _errors: &Effects,
        runtime: Duration,
        counts: &HashMap<Strategy, i32>,
    ) {
        println!(
            "           {:>10} {} {} {:?} {}",
            format_runtime(runtime),
            self.format_counts(counts),
            start.packed_string(),
            action.strategy(),
            action
        );
    }

    fn unsolved(
        &self,
        _givens: &str,
        start: &Board,
        _stopped: &Board,
        runtime: Duration,
        counts: &HashMap<Strategy, i32>,
    ) {
        println!(
            "unsolved   {:>10} {} {}",
            format_runtime(runtime),
            self.format_counts(counts),
            // givens,
            start.packed_string()
        );
    }

    fn solved(
        &self,
        _givens: &str,
        start: &Board,
        _solution: &Board,
        difficulty: Difficulty,
        runtime: Duration,
        counts: &HashMap<Strategy, i32>,
    ) {
        println!(
            "{:<10} {:>10} {} {}",
            format!("{:?}", difficulty),
            format_runtime(runtime),
            self.format_counts(counts),
            start.packed_string()
        );
    }
}
