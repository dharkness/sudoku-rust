//! Provides a text-based interface for creating and playing Sudoku puzzles.

use clap::Args;
use std::io::{stdout, Write};
use std::time::Instant;

use crate::build::Generator;
use crate::io::{
    format_for_fancy_console, format_for_wiki, format_grid, format_packed, format_runtime,
    print_candidate, print_candidates, Cancelable, Parse, SUDOKUWIKI_URL,
};
use crate::layout::{Cell, Known};
use crate::puzzle::{Board, Effects, Options};
use crate::solve::{find_brute_force, BruteForceResult, TECHNIQUES};
use crate::symbols::UNKNOWN_VALUE;

const MAXIMUM_SOLUTIONS: usize = 100;

#[derive(Debug, Args)]
#[clap(disable_help_flag = true)]
pub struct PlayArgs {
    /// Print help information
    #[clap(long, action = clap::ArgAction::HelpLong)]
    help: Option<bool>,

    /// Do not automatically remove peer candidates
    #[clap(short, long)]
    peers: bool,

    /// Do not automatically solve naked singles
    #[clap(short, long)]
    naked: bool,

    /// Do not automatically solve hidden singles
    #[clap(short, long)]
    hidden: bool,

    /// Do not automatically solve naked or hidden singles
    #[clap(short, long)]
    singles: bool,

    /// Clues for a starting puzzle
    puzzle: Option<String>,
}

impl PlayArgs {
    pub fn options(&self) -> Options {
        Options {
            remove_peers: !self.peers,
            solve_naked_singles: !self.naked && !self.singles,
            solve_hidden_singles: !self.hidden && !self.singles,
        }
    }
}

pub fn start_player(args: PlayArgs, cancelable: &Cancelable) {
    let mut options = args.options();
    let mut boards = vec![];
    let mut show_board = false;

    match args.puzzle {
        Some(clues) => {
            let parser = Parse::packed_with_options(true, options);
            let (board, effects, failure) = parser.parse(&clues);

            boards.push(board);
            if let Some((cell, known)) = failure {
                println!();
                print_candidates(&board);
                println!("\n==> Setting {} to {} will cause errors\n", cell, known);
                effects.print_errors();
                println!();
            } else {
                show_board = true;
            }
        }
        None => {
            boards.push(Board::new());
            print_help();
        }
    }

    loop {
        let board = boards.last().unwrap();
        if show_board {
            print_candidates(board);
            if board.is_solved() {
                println!("\n==> Congratulations!\n");
            } else {
                println!();
            }
            show_board = false;
        }

        print!(
            "[ {} solved - {} unsolved ] ",
            board.known_count(),
            board.unknown_count()
        );
        let _ = stdout().flush();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_uppercase();
        if input.is_empty() {
            continue;
        }
        let input = input.split(' ').collect::<Vec<_>>();

        match input[0] {
            "O" => {
                if input.len() >= 2 {
                    for c in input[1].to_uppercase().chars() {
                        match c {
                            'P' => {
                                options.remove_peers = !options.remove_peers;
                            }
                            'N' => {
                                options.solve_naked_singles = !options.solve_naked_singles;
                            }
                            'H' => {
                                options.solve_hidden_singles = !options.solve_hidden_singles;
                            }
                            _ => println!("\n==> Unknown option: {}", input[1].to_uppercase()),
                        }
                    }
                };
                println!(
                    concat!(
                        "\n==> Options\n",
                        "\n",
                        "  P - {} peer candidates\n",
                        "  N - {} naked singles\n",
                        "  H - {} hidden singles\n",
                    ),
                    if options.remove_peers {
                        "removing"
                    } else {
                        "not removing"
                    },
                    if options.solve_naked_singles {
                        "solving"
                    } else {
                        "not solving"
                    },
                    if options.solve_hidden_singles {
                        "solving"
                    } else {
                        "not solving"
                    },
                );
            }
            "N" => {
                if let Some(board) = create_new_puzzle(options) {
                    boards.push(board);
                    println!();
                }
            }
            "C" => {
                println!();
                let mut generator = Generator::new(false);
                match generator.generate(cancelable) {
                    Some(board) => {
                        println!("\n==> Clues: {}\n", board);
                        boards.push(board);
                    }
                    None => {
                        println!("\n==> Failed to create a new puzzle\n");
                    }
                }
                cancelable.clear();
            }
            "P" => {
                if input.len() >= 2 {
                    let k = input[1].chars().next().unwrap();
                    if ('1'..='9').contains(&k) {
                        println!();
                        print_candidate(board, Known::from(k));
                        println!();
                    } else {
                        println!("\n==> Invalid candidate \"{}\"\n", k);
                    }
                } else {
                    println!();
                    show_board = true
                }
            }
            "X" => {
                if input.len() >= 2 {
                    println!(
                        "\n==> {}\n",
                        format_packed(
                            board,
                            input[1].chars().next().unwrap_or(UNKNOWN_VALUE),
                            true
                        )
                    );
                } else {
                    println!("\n==> {}\n", format_for_fancy_console(board));
                };
            }
            "W" => {
                println!("\n==> {}{}\n", SUDOKUWIKI_URL, format_for_wiki(board));
            }
            "M" => {
                println!("\n{}\n", format_grid(board));
            }
            "G" => {
                if input.len() != 3 {
                    println!("\n==> G <cell> <digit>\n");
                    continue;
                }
                let cell = Cell::from(input[1].to_uppercase());
                let known = Known::from(input[2]);
                if !board.is_candidate(cell, known) {
                    println!("\n==> {} is not a candidate for {}\n", known, cell);
                    continue;
                }
                let mut clone = board.with_options(options);
                let mut effects = Effects::new();
                clone.set_given(cell, known, &mut effects);
                if effects.has_errors() {
                    println!("\n==> Invalid move\n");
                    effects.print_errors();
                    println!();
                    continue;
                }
                boards.push(clone);
                println!();
                show_board = true;
            }
            "S" => {
                if input.len() != 3 {
                    println!("\n==> S <cell> <digit>\n");
                    continue;
                }
                let cell = Cell::from(input[1].to_uppercase());
                let known = Known::from(input[2]);
                if !board.is_candidate(cell, known) {
                    println!("\n==> {} is not a candidate for {}\n", known, cell);
                    continue;
                }
                let mut clone = board.with_options(options);
                let mut effects = Effects::new();
                clone.set_known(cell, known, &mut effects);
                if effects.has_errors() {
                    println!("\n==> Invalid move\n");
                    effects.print_errors();
                    println!();
                    continue;
                }
                boards.push(clone);
                println!();
                show_board = true;
            }
            "E" => {
                if input.len() != 3 {
                    println!("\n==> E <cell> <digits>\n");
                    continue;
                }
                let cell = Cell::from(input[1]);
                let mut clone = board.with_options(options);
                let mut effects = Effects::new();
                for c in input[2].chars() {
                    let known = Known::from(c);
                    if !board.is_candidate(cell, known) {
                        println!("\n==> {} is not a candidate for {}\n", known, cell);
                        continue;
                    }
                    clone.remove_candidate(cell, known, &mut effects);
                }
                if effects.has_errors() {
                    println!("\n==> Invalid move\n");
                    effects.print_errors();
                    println!();
                    continue;
                }
                boards.push(clone);
                println!();
                show_board = true;
            }
            "V" => {
                let runtime = Instant::now();
                match find_brute_force(board, cancelable, false, 0, MAXIMUM_SOLUTIONS) {
                    BruteForceResult::AlreadySolved => {
                        println!("\n==> The puzzle is already solved\n");
                    }
                    BruteForceResult::TooFewKnowns => {
                        println!("\n==> The puzzle needs at least 17 solved cells to verify\n");
                    }
                    BruteForceResult::UnsolvableCells(cells) => {
                        println!("\n==> The puzzle cannot be solved with these {} empty cells\n\n    {}\n", cells.size(), cells);
                    }
                    BruteForceResult::Canceled => {
                        println!(
                            "\n==> The verification was canceled - took {} µs\n",
                            format_runtime(runtime.elapsed())
                        );
                        cancelable.clear();
                    }
                    BruteForceResult::Unsolvable => {
                        println!(
                            "\n==> The puzzle cannot be solved - took {} µs\n",
                            format_runtime(runtime.elapsed())
                        );
                    }
                    BruteForceResult::Solved(_) => {
                        println!(
                            "\n==> The puzzle is solvable - took {} µs\n",
                            format_runtime(runtime.elapsed())
                        );
                    }
                    BruteForceResult::MultipleSolutions(solutions) => {
                        println!(
                            "\n==> The puzzle has {}{} solutions - took {} µs\n",
                            if solutions.len() > MAXIMUM_SOLUTIONS {
                                "at least "
                            } else {
                                ""
                            },
                            solutions.len(),
                            format_runtime(runtime.elapsed())
                        );
                    }
                }
            }
            "F" => {
                let mut found = false;
                TECHNIQUES.iter().for_each(|solver| {
                    if let Some(effects) = solver.solve(board) {
                        found = true;
                        println!(
                            "\n==> Found {}\n",
                            pluralize(effects.action_count(), solver.name())
                        );
                        effects.print_actions();
                    }
                });

                if !found {
                    println!("\n==> No deductions found\n");
                } else {
                    println!();
                }
            }
            "A" => {
                let mut any_applied = false;
                let mut clone = board.with_options(options);
                let _ = TECHNIQUES.iter().try_for_each(|solver| {
                    if let Some(deductions) = solver.solve(board) {
                        let mut applied = 0;
                        for deduction in deductions.actions() {
                            let mut next = clone;
                            deduction.apply(&mut clone, &mut Effects::new());
                            if let Some(errors) = deductions.apply_all(&mut next) {
                                println!(
                                    "\n==> Applying {} will cause errors\n    {}\n\n",
                                    solver.name(),
                                    deduction
                                );
                                errors.print_errors();
                                return Err(());
                            }
                            applied += 1;
                            clone = next;
                        }
                        if applied > 0 {
                            any_applied = true;
                            println!("\n==> Applied {}", pluralize(applied, solver.name()));
                        }
                    }
                    Ok(())
                });

                if any_applied {
                    boards.push(clone);
                    println!();
                    show_board = true;
                } else {
                    println!("\n==> No deductions applied\n");
                }
            }
            "R" => {
                let mut reset = Board::new();
                for (cell, known) in board.known_iter() {
                    let mut effects = Effects::new();
                    reset.set_given(cell, known, &mut effects);
                    if effects.has_errors() {
                        println!("\n==> Invalid board\n");
                        effects.print_errors();
                    }
                }
                boards.push(reset);
                println!();
                show_board = true;
            }
            "Z" => {
                if boards.len() > 1 {
                    println!("\n==> Undoing last move\n");
                    boards.pop();
                    show_board = true
                }
            }
            "?" | "H" => print_help(),
            "Q" => break,
            _ => println!("\n==> Unknown command: {}\n", input[0]),
        }
    }
}

fn print_help() {
    println!(concat!(
        "\n==> Help\n",
        "\n",
        "  O [option]        - view or toggle an option\n",
        "  N                 - start or input a new puzzle\n",
        "  C                 - create a new random puzzle\n",
        "\n",
        "  P <digit>         - print the puzzle, optionally limited to a single candidate\n",
        "  X [char]          - export the puzzle with optional character for unsolved cells\n",
        "  W                 - print URL to play on SudokuWiki.org\n",
        "  M                 - print the puzzle as a grid suitable for email\n",
        "\n",
        "  G <cell> <digit>  - set the given (clue) for a cell\n",
        "  S <cell> <digit>  - solve a cell\n",
        "  E <cell> <digits> - erase one or more candidates\n",
        "\n",
        "  V                 - verify puzzle is solvable\n",
        "  F                 - find deductions\n",
        "  A                 - apply deductions\n",
        "  B                 - use Bowman's Bingo to solve the puzzle if possible\n",
        "  R                 - reset candidates based on solved cells\n",
        "  Z                 - undo last change\n",
        "\n",
        "  H                 - this help message\n",
        "  Q                 - quit\n",
        "\n",
        "      <option> - P, N or H\n",
        "      <cell>   - A1 to J9\n",
        "      <digit>  - 1 to 9\n",
        "      <char>   - any single character\n",
        "\n",
        "  Note: commands and cells are not case-sensitive\n",
    ))
}

fn create_new_puzzle(options: Options) -> Option<Board> {
    println!(concat!(
        "\n==> Enter the givens\n\n",
        "  - enter up to 81 digits\n",
        "  - use period or zero to leave a cell blank\n",
        "  - spaces are ignored\n",
        "  - leave empty to cancel\n",
        "  - enter 'E' for an empty puzzle\n",
    ));

    loop {
        print!("> ");
        let _ = stdout().flush();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().replace(' ', "").to_uppercase();
        if input.is_empty() {
            println!();
            return None;
        }
        if input == "E" {
            let board = Board::new();

            println!();
            print_candidates(&board);
            return Some(board);
        }
        if input.len() > 81 {
            println!(
                concat!(
                    "\n==> Expected at most 81 digits, got {}\n\n",
                    "{}\n",
                    "        |        |        |        |        |        |        |        |        |\n",
                ),
                input.len(),
                input
            );
            continue;
        }

        let parser = Parse::packed_with_options(true, options);
        let (board, effects, failure) = parser.parse(&input);

        if let Some((cell, known)) = failure {
            println!();
            print_candidates(&board);
            println!("\n==> Setting {} to {} will cause errors\n", cell, known);
            effects.print_errors();
        } else {
            println!();
            print_candidates(&board);
        }

        return Some(board);
    }
}

fn pluralize(count: usize, label: &str) -> String {
    if count == 1 {
        format!("{} {}", count, label)
    } else if ES_SUFFIXES.iter().any(|suffix| label.ends_with(suffix)) {
        format!("{} {}es", count, label)
    } else {
        format!("{} {}s", count, label)
    }
}

const ES_SUFFIXES: [&str; 1] = ["sh"];
