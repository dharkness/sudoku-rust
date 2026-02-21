use std::collections::HashMap;
use std::io::BufRead;
use std::time::{Duration, Instant};

use clap::Args;
use itertools::Itertools;

use crate::io::{
    format_for_wiki, format_number, format_runtime, print_all_and_single_candidates,
    print_solved_values, Cancelable, Parse, ParsePacked, Parser, SUDOKUWIKI_URL,
};
use crate::layout::{Cell, Digit};
use crate::puzzle::{Action, Board, Changer, Difficulty, Effects, Options, Strategy};
use crate::solve::{Reporter, Resolution, Solver, Timings, NON_PEER_TECHNIQUES};

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
    let mut count = 0;
    let mut solved = 0;
    let runtime = Instant::now();

    match args.puzzles {
        Some(puzzles) => {
            let reporter = DetailedReporter::new();
            let mut parser_solver = ParserSolver::new(&parser, &solver, &reporter, &mut timings);

            for puzzle in puzzles {
                if cancelable.is_canceled() {
                    break;
                }
                if parser_solver.parse_and_solve(&puzzle) {
                    solved += 1;
                }
                count += 1;
            }
        }
        None => {
            let reporter = TableReporter::new();
            let mut parser_solver = ParserSolver::new(&parser, &solver, &reporter, &mut timings);
            let stdin = std::io::stdin();

            print_table_header(3);
            for puzzle in stdin.lock().lines().map_while(Result::ok) {
                if cancelable.is_canceled() {
                    break;
                }
                if parser_solver.parse_and_solve(&puzzle) {
                    solved += 1;
                }
                count += 1;
            }
        }
    }

    let totals = timings.strategy_totals();

    println!(
        "\nsolved {} of {} puzzles in {} µs",
        format_number(solved),
        format_number(count),
        format_runtime(runtime.elapsed())
    );

    println!();
    print_table_header(5);
    println!(
        "{:<10} {:>10} {}",
        "Total",
        format_runtime(runtime.elapsed()),
        format_counts(&totals, 5)
    );

    println!();
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

        if let Some((cell, digit)) = failure {
            self.reporter
                .invalid(givens, &start, &effects, cell, digit, runtime.elapsed());
            return false;
        }

        match self.solver.solve(&start, &effects, self.timings) {
            Resolution::Canceled(..) => false,
            Resolution::Failed(board, applied, _, action, errors) => {
                self.reporter.failed(
                    givens,
                    &start,
                    &board,
                    &action,
                    &errors,
                    runtime.elapsed(),
                    &applied.action_counts(),
                );
                false
            }
            Resolution::Unsolved(board, applied, _) => {
                self.reporter.unsolved(
                    givens,
                    &start,
                    &board,
                    runtime.elapsed(),
                    &applied.action_counts(),
                );
                false
            }
            Resolution::Solved(solution, actions, difficulty) => {
                self.reporter.solved(
                    givens,
                    &start,
                    &solution,
                    difficulty,
                    runtime.elapsed(),
                    &actions.action_counts(),
                );
                true
            }
        }
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
                println!("- {:>2} {}", count, strategy.label());
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
        digit: Digit,
        runtime: Duration,
    ) {
        println!("invalid in {} µs\n", format_runtime(runtime));
        print_all_and_single_candidates(partial);
        println!("\nsetting {} to {} will cause errors\n", cell, digit);
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
        print_solved_values(solution);
        println!();
        self.print_counts(counts);
        println!();
    }
}

struct TableReporter {}

impl TableReporter {
    fn new() -> TableReporter {
        TableReporter {}
    }
}

impl Reporter for TableReporter {
    fn invalid(
        &self,
        givens: &str,
        _partial: &Board,
        _errors: &Effects,
        cell: Cell,
        digit: Digit,
        _runtime: Duration,
    ) {
        eprintln!("invalid: cannot set {} to {} for {}", cell, digit, givens);
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
            format_counts(counts, 3),
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
            format_counts(counts, 3),
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
            format_counts(counts, 3),
            start.packed_string()
        );
    }
}

fn dash_zero(value: i32) -> String {
    if value == 0 {
        "-".to_string()
    } else {
        value.to_string()
    }
}

fn print_table_header(width: usize) {
    print!("                   µs");
    let header = NON_PEER_TECHNIQUES
        .iter()
        .map(|solver| format_column(solver.acronym(), width))
        .collect::<Vec<_>>()
        .join(" ");
    if !header.is_empty() {
        print!(" {}", header);
    }
    println!();
}

fn format_counts(counts: &HashMap<Strategy, i32>, width: usize) -> String {
    NON_PEER_TECHNIQUES
        .iter()
        .map(|technique| dash_zero(*counts.get(&technique.strategy()).unwrap_or(&0)))
        .map(|value| format_column(&value, width))
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_column(value: &str, width: usize) -> String {
    let inner = width.saturating_sub(1);
    format!("{:>inner$}", value, inner = inner)
}
