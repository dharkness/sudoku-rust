//! Provides a text-based interface for creating and playing Sudoku puzzles.

use clap::Args;
use std::io::{stdout, Write};
use std::time::Instant;

use crate::build::{Finder, Generator};
use crate::io::{
    format_for_fancy_console, format_for_wiki, format_grid, format_packed, format_runtime,
    print_candidate, print_candidates, print_givens, print_known_values, Cancelable, Parse,
    SUDOKUWIKI_URL,
};
use crate::layout::{Cell, Known};
use crate::puzzle::{Board, Change, Changer, Effects, Options, Strategy};
use crate::solve::{find_brute_force, BruteForceResult, NON_PEER_TECHNIQUES};
use crate::symbols::{MISSING, UNKNOWN_VALUE};

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

    /// Do not automatically solve intersection removals
    #[clap(short, long)]
    intersection: bool,

    /// Clues for a starting puzzle
    puzzle: Option<String>,
}

impl PlayArgs {
    pub fn options(&self) -> Options {
        Options {
            stop_on_error: true,
            remove_peers: !self.peers,
            solve_naked_singles: !self.naked && !self.singles,
            solve_hidden_singles: !self.hidden && !self.singles,
            solve_intersection_removals: !self.intersection,
        }
    }
}

pub fn start_player(args: PlayArgs, cancelable: &Cancelable) {
    let mut changer = Changer::new(args.options());
    let mut boards = vec![];
    let mut show_board = false;
    let mut deductions = None;

    match args.puzzle {
        Some(clues) => {
            let parser = Parse::packed_with_player(changer);
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
            show_board = false;
            if board.is_fully_solved() {
                print_known_values(board);
                println!("\n==> Congratulations!\n");
            } else {
                print_candidates(board);
                println!();
            }
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
                                changer.options.remove_peers = !changer.options.remove_peers;
                            }
                            'N' => {
                                changer.options.solve_naked_singles =
                                    !changer.options.solve_naked_singles;
                            }
                            'H' => {
                                changer.options.solve_hidden_singles =
                                    !changer.options.solve_hidden_singles;
                            }
                            'I' => {
                                changer.options.solve_intersection_removals =
                                    !changer.options.solve_intersection_removals;
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
                        "  I - {} intersection removals\n",
                    ),
                    if changer.options.remove_peers {
                        "removing"
                    } else {
                        "not removing"
                    },
                    if changer.options.solve_naked_singles {
                        "solving"
                    } else {
                        "not solving"
                    },
                    if changer.options.solve_hidden_singles {
                        "solving"
                    } else {
                        "not solving"
                    },
                    if changer.options.solve_intersection_removals {
                        "solving"
                    } else {
                        "not solving"
                    },
                );
            }
            "N" => {
                if let Some(board) = create_new_puzzle(changer) {
                    deductions = None;
                    boards.push(board);
                    println!();
                }
            }
            "C" => {
                println!();
                let mut generator = Generator::new(false, true);
                match generator.generate(&changer, cancelable) {
                    Some(board) => {
                        let mut finder = Finder::new(22, 10, true);
                        let (start, _) = finder.backtracking_find(board, cancelable);
                        println!("\n==> Clues: {}\n", start);
                        deductions = None;
                        boards.push(start);
                        show_board = true;
                    }
                    None => {
                        println!("\n==> Failed to create a new puzzle\n");
                    }
                }
                cancelable.clear();
            }

            "P" => {
                if input.len() >= 2 {
                    let c = input[1].to_uppercase().chars().next().unwrap();
                    if c == 'G' {
                        println!();
                        print_givens(board);
                        println!();
                    } else if c == 'K' {
                        println!();
                        print_known_values(board);
                        println!();
                    } else if ('1'..='9').contains(&c) {
                        println!();
                        print_candidate(board, Known::from(c));
                        println!();
                    } else {
                        println!("\n==> Invalid candidate \"{}\"\n", c);
                    }
                } else if board.is_fully_solved() {
                    println!();
                    print_known_values(board);
                    println!();
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
                match changer.set_given(board, Strategy::Given, cell, known) {
                    Change::None => {
                        println!("\n==> {} is not a candidate for {}\n", known, cell);
                        continue;
                    }
                    Change::Valid(after, _) => {
                        deductions = None;
                        boards.push(*after);
                        println!();
                        show_board = true;
                    }
                    Change::Invalid(_, _, _, errors) => {
                        println!("\n==> Invalid move\n");
                        errors.print_errors();
                        println!();
                        continue;
                    }
                }
            }
            "S" => {
                if input.len() != 3 {
                    println!("\n==> S <cell> <digit>\n");
                    continue;
                }
                let cell = Cell::from(input[1].to_uppercase());
                let known = Known::from(input[2]);
                match changer.set_known(board, Strategy::Solve, cell, known) {
                    Change::None => {
                        println!("\n==> {} is not a candidate for {}\n", known, cell);
                        continue;
                    }
                    Change::Valid(after, _) => {
                        deductions = None;
                        boards.push(*after);
                        println!();
                        show_board = true;
                    }
                    Change::Invalid(_, _, _, errors) => {
                        println!("\n==> Invalid move\n");
                        errors.print_errors();
                        println!();
                        continue;
                    }
                }
            }
            "E" => {
                if input.len() != 3 {
                    println!("\n==> E <cell> <digits>\n");
                    continue;
                }
                let cell = Cell::from(input[1]);
                let mut clone = *board;
                let mut changed = false;
                for c in input[2].chars() {
                    let known = Known::from(c);
                    match changer.remove_candidate(&clone, Strategy::Erase, cell, known) {
                        Change::None => {
                            println!("\n==> {} is not a candidate for {}\n", known, cell);
                            continue;
                        }
                        Change::Valid(after, _) => {
                            clone = *after;
                            changed = true;
                        }
                        Change::Invalid(_, _, _, errors) => {
                            println!("\n==> Invalid move\n");
                            errors.print_errors();
                            println!();
                            continue;
                        }
                    }
                }
                if changed {
                    deductions = None;
                    boards.push(clone);
                    println!();
                    show_board = true;
                }
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
                        println!("\n==> The puzzle cannot be solved with these {} empty cells\n\n    {}\n", cells.len(), cells);
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
                };
            }
            "F" => {
                if deductions.is_none() {
                    let mut found = Effects::new();
                    NON_PEER_TECHNIQUES.iter().for_each(|solver| {
                        if let Some(actions) = solver.solve(board) {
                            found.take_actions(actions);
                        }
                    });
                    deductions = Some(found);
                }

                let mut affecting_cell = None;
                if input.len() == 2 {
                    affecting_cell = Some(Cell::from(input[1].to_uppercase()));
                };

                if let Some(ref found) = deductions {
                    if let Some(cell) = affecting_cell {
                        let filtered = found.affecting_cell(cell);
                        if filtered.is_empty() {
                            println!("\n==> No deductions found affecting {}\n", cell);
                            continue;
                        }
                        println!(
                            "\n==> Found {} affecting {}\n",
                            pluralize(filtered.action_count(), "deduction"),
                            cell
                        );
                    } else {
                        println!(
                            "\n==> Found {}\n",
                            pluralize(found.action_count(), "deduction")
                        );
                    }
                    let mut found_any = false;
                    for (i, action) in found.actions().iter().enumerate() {
                        if let Some(cell) = affecting_cell {
                            if action.affects_cell(cell) {
                                found_any = true;
                                println!("{:>4} - {}", i + 1, action);
                            }
                        } else {
                            found_any = true;
                            println!("{:>4} - {}", i + 1, action);
                        }
                    }
                    if found_any {
                        println!();
                    }
                } else if let Some(cell) = affecting_cell {
                    println!("\n==> No deductions found affecting {}\n", cell);
                } else {
                    println!("\n==> No deductions found\n");
                }
            }
            "A" => {
                if input.len() >= 2 {
                    if let Some(ref mut found) = &mut deductions {
                        let n = input[1].parse::<usize>().unwrap_or(0);
                        if n < 1 || n > found.action_count() {
                            println!(
                                "\n==> Enter a deduction number 1 - {}\n",
                                found.action_count()
                            );
                            continue;
                        }
                        let deduction = &found.actions()[n - 1];
                        match changer.apply(board, deduction) {
                            Change::None => {
                                println!("\n==> Did not apply {}\n", deduction);
                            }
                            Change::Valid(after, _) => {
                                boards.push(*after);
                                println!("\n==> Applied {}\n", deduction);
                                deductions = None;
                                show_board = true;
                            }
                            Change::Invalid(_, _, _, errors) => {
                                println!("\n==> Applying {} will cause errors\n", deduction);
                                errors.print_errors();
                                println!();
                            }
                        }
                    } else {
                        println!("\n==> Find deductions first with F\n");
                    }
                    continue;
                }

                let mut any_applied = false;
                let mut clone = *board;
                let _ = NON_PEER_TECHNIQUES.iter().try_for_each(|solver| {
                    if let Some(actions) = solver.solve(board) {
                        let mut applied = 0;
                        for action in actions.actions() {
                            match changer.apply(&clone, action) {
                                Change::None => (),
                                Change::Valid(after, _) => {
                                    applied += 1;
                                    clone = *after;
                                }
                                Change::Invalid(_, _, _, errors) => {
                                    println!(
                                        "\n==> Applying {} will cause errors\n    {}\n",
                                        solver.name(),
                                        action
                                    );
                                    errors.print_errors();
                                    return Err(());
                                }
                            }
                        }
                        if applied > 0 {
                            any_applied = true;
                            println!("\n==> Applied {}", pluralize(applied, solver.name()));
                        }
                    }
                    Ok(())
                });

                if any_applied {
                    deductions = None;
                    boards.push(clone);
                    println!();
                    show_board = true;
                } else {
                    println!("\n==> No deductions applied\n");
                }
            }
            "B" => {
                let runtime = Instant::now();
                match find_brute_force(board, cancelable, false, 0, MAXIMUM_SOLUTIONS) {
                    BruteForceResult::AlreadySolved => {
                        println!("\n==> The puzzle is already solved\n");
                    }
                    BruteForceResult::TooFewKnowns => {
                        println!("\n==> The puzzle needs at least 17 solved cells to verify\n");
                    }
                    BruteForceResult::UnsolvableCells(cells) => {
                        println!("\n==> The puzzle cannot be solved with these {} empty cells\n\n    {}\n", cells.len(), cells);
                    }
                    BruteForceResult::Canceled => {
                        println!(
                            "\n==> The solution was canceled - took {} µs\n",
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
                    BruteForceResult::Solved(actions) => {
                        println!(
                            "\n==> The puzzle was solved - took {} µs",
                            format_runtime(runtime.elapsed())
                        );
                        match changer.apply_all(board, &actions) {
                            Change::None => (),
                            Change::Valid(after, _) => {
                                boards.push(*after);
                                println!();
                                show_board = true;
                            }
                            Change::Invalid(_, _, _, errors) => {
                                println!("\n==> Solution caused errors\n");
                                errors.print_errors();
                                println!();
                                continue;
                            }
                        }
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
                };
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
                deductions = None;
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
        "  O [option]          - view or toggle an option\n",
        "  N                   - start or input a new puzzle\n",
        "  C                   - create a new random puzzle\n",
        "\n",
        "  P [G | K | digit]   - print the full puzzle, givens, knowns, or a single candidate\n",
        "  X [char]            - export the puzzle with optional character for unsolved cells\n",
        "  W                   - print URL to play on SudokuWiki.org\n",
        "  M                   - print the puzzle as a grid suitable for email\n",
        "\n",
        "  G <cell> <digit>    - set the given (clue) for a cell\n",
        "  S <cell> <digit>    - solve a cell\n",
        "  E <cell> <digits>   - erase one or more candidates\n",
        "\n",
        "  V                   - verify puzzle is solvable\n",
        "  F                   - find deductions\n",
        "  A <num>             - apply a single or all deductions\n",
        "  B                   - use Bowman's Bingo to solve the puzzle if possible\n",
        "  R                   - reset candidates based on solved cells\n",
        "  Z                   - undo last change\n",
        "\n",
        "  H                   - this help message\n",
        "  Q                   - quit\n",
        "\n",
        "      <option> - P, N or H\n",
        "      <cell>   - A1 to J9\n",
        "      <digit>  - 1 to 9\n",
        "      <num>    - any positive number\n",
        "      <char>   - any single character\n",
        "\n",
        "  Commands and cells are not case-sensitive - \"s a2 4\" and \"E D8 6\" are fine\n",
    ))
}

fn create_new_puzzle(changer: Changer) -> Option<Board> {
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
        let input = input
            .trim()
            .replace(' ', "")
            .replace(MISSING, ".")
            .to_uppercase();
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

        let parser = Parse::packed_with_player(changer);
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
