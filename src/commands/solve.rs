use std::collections::HashMap;
use std::io::BufRead;
use std::time::{Duration, Instant};

use clap::Args;
use itertools::Itertools;

use crate::io::{
    format_for_wiki, format_number, format_runtime, print_all_and_single_candidates,
    print_known_values, Cancelable, Parse, ParsePacked, Parser, SUDOKUWIKI_URL,
};
use crate::layout::{Cell, Known};
use crate::puzzle::{Action, Board, Changer, Difficulty, Effects, Options, Strategy};
use crate::solve::{Reporter, Resolution, Solver, Timings};

#[derive(Debug, Args)]
pub struct SolveArgs {
    /// Check the results of each solver strategy using brute force
    #[clap(short, long)]
    check: bool,

    /// Clues for one or more puzzles to solve with detailed output
    puzzles: Option<Vec<String>>,
}

/// Creates a new puzzle and prints it to stdout.
pub fn solve_puzzles(args: SolveArgs) {
    let cancelable = Cancelable::new();
    let changer = Changer::new(Options::errors());
    let parser = Parse::packed_with_player(changer);
    let solver = Solver::new(args.check);
    let mut timings = Timings::new();

    match args.puzzles {
        Some(puzzles) => {
            let reporter = DetailedReporter::new();
            let mut parser_solver = ParserSolver::new(&parser, &solver, &reporter, &mut timings);

            for puzzle in puzzles {
                parser_solver.parse_and_solve(&puzzle);
                if cancelable.is_canceled() {
                    break;
                }
            }
        }
        None => {
            let reporter = CSVReporter::new();
            let mut parser_solver = ParserSolver::new(&parser, &solver, &reporter, &mut timings);
            let stdin = std::io::stdin();

            let runtime = Instant::now();
            let mut count = 0;
            let mut solved = 0;

            println!("                   µs NS HS NP NT NQ HP HT HQ PP PT BL XW SC YW ER SF XZ JF SK TS AR XY UR AU FW EU HU WZ BG");
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
                "\nsolved {} of {} puzzles in {} µs\n",
                format_number(solved),
                format_number(count),
                format_runtime(runtime.elapsed())
            );
        }
    }

    timings.print_details();
    println!();
    timings.print_totals();
}

struct ParserSolver<'a> {
    parser: &'a ParsePacked,
    solver: &'a Solver,
    reporter: &'a dyn Reporter,
    timings: &'a mut Timings,
}

impl ParserSolver<'_> {
    fn new<'a>(
        parser: &'a ParsePacked,
        solver: &'a Solver,
        reporter: &'a dyn Reporter,
        timings: &'a mut Timings,
    ) -> ParserSolver<'a> {
        ParserSolver {
            parser,
            solver,
            reporter,
            timings,
        }
    }

    fn parse_and_solve(&mut self, givens: &str) -> bool {
        let runtime = Instant::now();
        let (start, effects, failure) = self.parser.parse(givens);

        if let Some((cell, known)) = failure {
            self.reporter
                .invalid(givens, &start, &effects, cell, known, runtime.elapsed());
            return false;
        }

        match self.solver.solve(&start, &effects, self.timings) {
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
        print_all_and_single_candidates(partial);
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
        print_all_and_single_candidates(stopped);
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
        print_all_and_single_candidates(stopped);
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
        print_known_values(solution);
        println!();
        self.print_counts(counts);
        println!();
    }
}

struct CSVReporter {}

impl CSVReporter {
    fn new() -> CSVReporter {
        CSVReporter {}
    }

    fn format_counts(&self, counts: &HashMap<Strategy, i32>) -> String {
        format!(
            "{:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2} {:>2}",
            // counts.get(&Strategy::Peer).unwrap_or(0),
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
            counts.get(&Strategy::EmptyRectangle).unwrap_or(&0),
            counts.get(&Strategy::Swordfish).unwrap_or(&0),
            counts.get(&Strategy::XYZWing).unwrap_or(&0),

            counts.get(&Strategy::Jellyfish).unwrap_or(&0),
            counts.get(&Strategy::Skyscraper).unwrap_or(&0),
            counts.get(&Strategy::AvoidableRectangle).unwrap_or(&0),
            counts.get(&Strategy::TwoStringKite).unwrap_or(&0),
            counts.get(&Strategy::XYChain).unwrap_or(&0),
            counts.get(&Strategy::UniqueRectangle).unwrap_or(&0),
            counts.get(&Strategy::AlmostUniqueRectangle).unwrap_or(&0),
            counts.get(&Strategy::Fireworks).unwrap_or(&0),
            counts.get(&Strategy::ExtendedUniqueRectangle).unwrap_or(&0),
            counts.get(&Strategy::HiddenUniqueRectangle).unwrap_or(&0),
            counts.get(&Strategy::WXYZWing).unwrap_or(&0),
            counts.get(&Strategy::Bug).unwrap_or(&0),
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
        _action: &Action,
        _errors: &Effects,
        runtime: Duration,
        counts: &HashMap<Strategy, i32>,
    ) {
        println!(
            "Invalid    {:>10} {} {}",
            format_runtime(runtime),
            self.format_counts(counts),
            start.packed_string()
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
            "Unsolved   {:>10} {} {}",
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
