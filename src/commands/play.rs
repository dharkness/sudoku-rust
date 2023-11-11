//! Provides a text-based interface for creating and playing Sudoku puzzles.

use std::io::{stdout, Write};
use std::time::Instant;

use clap::Args;

use crate::build::{Finder, Generator};
use crate::io::{
    format_for_fancy_console, format_for_wiki, format_grid, format_packed, format_runtime,
    print_all_and_single_candidates, print_all_and_single_candidates_with_highlight,
    print_candidate, print_givens, print_known_values, Cancelable, Parse, Parser, SUDOKUWIKI_URL,
};
use crate::layout::{Cell, CellSet, Known, KnownSet};
use crate::puzzle::{Board, ChangeResult, Changer, Effects, Options, Strategy};
use crate::solve::{find_brute_force, BruteForceResult, TECHNIQUES};
use crate::symbols::{MISSING, UNKNOWN_VALUE};

const MAXIMUM_SOLUTIONS: usize = 100;

#[derive(Debug, Args)]
#[clap(disable_help_flag = true)]
pub struct PlayArgs {
    /// Print help information
    #[clap(long, action = clap::ArgAction::HelpLong)]
    help: Option<bool>,

    /// Automatically solve naked singles
    #[clap(short, long)]
    naked: bool,

    /// Automatically solve hidden singles
    #[clap(short, long)]
    hidden: bool,

    /// Automatically solve naked or hidden singles (same as --naked --hidden)
    #[clap(short, long)]
    singles: bool,

    /// Automatically solve intersection removals
    #[clap(short, long)]
    intersection: bool,

    /// Clues for a starting puzzle
    puzzle: Option<String>,
}

impl PlayArgs {
    pub fn new() -> Self {
        Self {
            help: None,
            naked: false,
            hidden: false,
            singles: false,
            intersection: false,
            puzzle: None,
        }
    }

    pub fn options(&self) -> Options {
        Options {
            stop_on_error: true,
            solve_naked_singles: self.naked || self.singles,
            solve_hidden_singles: self.hidden || self.singles,
            solve_intersection_removals: self.intersection,
        }
    }
}

pub fn start_player(args: PlayArgs) {
    let cancelable = Cancelable::new();
    let mut changer = Changer::new(args.options());
    let mut boards = vec![];
    let mut show_board = false;
    let mut deductions = None;
    let mut highlight = None;

    match args.puzzle {
        Some(clues) => {
            let parser = Parse::packed_with_player(changer);
            let (board, effects, failure) = parser.parse(&clues);

            boards.push(board);
            if let Some((cell, known)) = failure {
                println!();
                print_all_and_single_candidates(&board);
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
            } else if let Some(action) = &highlight {
                print_all_and_single_candidates_with_highlight(board, action);
                println!();
            } else {
                print_all_and_single_candidates(board);
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
                        "  N - {} naked singles\n",
                        "  H - {} hidden singles\n",
                        "  I - {} intersection removals\n",
                    ),
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
                    highlight = None;
                    boards.push(board);
                    println!();
                }
            }
            "C" => {
                println!();
                let mut generator = Generator::new(false, true);
                match generator.generate(&changer) {
                    Some(board) => {
                        let mut finder = Finder::new(22, 10, true);
                        let (start, _) = finder.backtracking_find(board);
                        println!("\n==> Clues: {}\n", start);
                        deductions = None;
                        highlight = None;
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
                        print_candidate(board, Known::from_char(c));
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
                    println!("\n==> G <cells> <digit>\n");
                    continue;
                }
                let cells = CellSet::from(input[1]);
                let known = match Known::try_from(input[2]) {
                    Ok(known) => known,
                    Err(message) => {
                        println!("\n==> {}\n", message);
                        continue;
                    }
                };
                let mut changed = false;
                let mut clone = *board;
                for cell in cells {
                    match changer.set_given(&clone, Strategy::Given, cell, known) {
                        ChangeResult::None => {
                            println!("\n==> {} is not a candidate for {}\n", known, cell);
                        }
                        ChangeResult::Valid(after, _) => {
                            clone = *after;
                            changed = true;
                        }
                        ChangeResult::Invalid(_, _, _, errors) => {
                            println!("\n==> Invalid move\n");
                            errors.print_errors();
                        }
                    }
                }
                if changed {
                    deductions = None;
                    highlight = None;
                    boards.push(clone);
                    println!();
                    show_board = true;
                }
            }
            "S" => {
                if input.len() != 3 {
                    println!("\n==> S <cells> <digit>\n");
                    continue;
                }
                let cells = CellSet::from(input[1]);
                let known = match Known::try_from(input[2]) {
                    Ok(known) => known,
                    Err(message) => {
                        println!("\n==> {}\n", message);
                        continue;
                    }
                };
                let mut clone = *board;
                let mut changed = false;
                for cell in cells {
                    match changer.set_known(&clone, Strategy::Solve, cell, known) {
                        ChangeResult::None => {
                            println!("\n==> {} is not a candidate for {}\n", known, cell);
                        }
                        ChangeResult::Valid(after, _) => {
                            clone = *after;
                            changed = true;
                        }
                        ChangeResult::Invalid(_, _, _, errors) => {
                            println!("\n==> Invalid move\n");
                            errors.print_errors();
                            println!();
                        }
                    }
                }
                if changed {
                    deductions = None;
                    highlight = None;
                    boards.push(clone);
                    println!();
                    show_board = true;
                }
            }
            "E" => {
                if input.len() != 3 {
                    println!("\n==> E <cells> <digits>\n");
                    continue;
                }
                let cells = CellSet::from(input[1]);
                let mut clone = *board;
                let mut changed = false;
                for cell in cells {
                    for known in KnownSet::from(input[2]) {
                        match changer.remove_candidate(&clone, Strategy::Erase, cell, known) {
                            ChangeResult::None => {
                                println!("\n==> {} is not a candidate for {}", known, cell);
                            }
                            ChangeResult::Valid(after, _) => {
                                clone = *after;
                                changed = true;
                            }
                            ChangeResult::Invalid(_, _, _, errors) => {
                                println!("\n==> Invalid move\n");
                                errors.print_errors();
                                println!();
                            }
                        }
                    }
                }
                if changed {
                    deductions = None;
                    highlight = None;
                    boards.push(clone);
                    println!();
                    show_board = true;
                }
            }

            "V" => {
                let runtime = Instant::now();
                match find_brute_force(board, false, 0, MAXIMUM_SOLUTIONS) {
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
                    TECHNIQUES.iter().for_each(|solver| {
                        if let Some(actions) = solver.solve(board) {
                            found.take_actions(actions);
                        }
                    });
                    deductions = Some(found);
                }

                let mut affecting_cell = None;
                let mut affecting_known = None;
                if input.len() == 2 {
                    match input[1].len() {
                        1 => {
                            if let Ok(known) = Known::try_from(input[1]) {
                                affecting_known = Some(known);
                            } else {
                                println!("\n==> Invalid digit: {}\n", input[1]);
                                continue;
                            }
                        }
                        2 => {
                            if let Ok(cell) = Cell::try_from(input[1]) {
                                affecting_cell = Some(cell);
                            } else {
                                println!("\n==> Invalid cell: {}\n", input[1]);
                                continue;
                            }
                        }
                        _ => (),
                    }
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
                    } else if let Some(known) = affecting_known {
                        let filtered = found.affecting_known(known);
                        if filtered.is_empty() {
                            println!("\n==> No deductions found affecting {}\n", known);
                            continue;
                        }
                        println!(
                            "\n==> Found {} affecting {}\n",
                            pluralize(filtered.action_count(), "deduction"),
                            known
                        );
                    } else {
                        println!(
                            "\n==> Found {}\n",
                            pluralize(found.action_count(), "deduction")
                        );
                    }

                    let mut found_any = false;
                    for (i, action) in found.actions().iter().enumerate() {
                        let mut found = None;
                        if let Some(cell) = affecting_cell {
                            if action.affects_cell(cell) {
                                found = Some(action);
                            }
                        } else if let Some(known) = affecting_known {
                            if action.affects_known(known) {
                                found = Some(action);
                            }
                        } else {
                            found = Some(action);
                        }
                        if let Some(action) = found {
                            found_any = true;
                            println!("{:>4} - {}", i + 1, action);
                            // if action.has_clues() {
                            //     println!("                           {}", action.clues());
                            // }
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
            "H" => {
                if input.len() != 2 {
                    println!("\n==> H <num>\n");
                    continue;
                }
                if let Some(ref mut found) = &mut deductions {
                    let n = input[1].parse::<usize>().unwrap_or(0);
                    if n < 1 || n > found.action_count() {
                        println!(
                            "\n==> Enter a deduction number 1 - {}\n",
                            found.action_count()
                        );
                        continue;
                    }
                    let action = found.actions()[n - 1].clone();
                    println!(
                        "\n==> Highlighting deduction {} - {:?}",
                        n,
                        action.strategy()
                    );
                    highlight = Some(action);
                    println!();
                    show_board = true;
                } else {
                    println!("\n==> Find deductions first with F\n");
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
                            ChangeResult::None => {
                                println!("\n==> Did not apply {}\n", deduction);
                            }
                            ChangeResult::Valid(after, _) => {
                                boards.push(*after);
                                println!("\n==> Applied {}\n", deduction);
                                deductions = None;
                                highlight = None;
                                show_board = true;
                            }
                            ChangeResult::Invalid(_, _, _, errors) => {
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
                let _ = TECHNIQUES.iter().try_for_each(|solver| {
                    if let Some(actions) = solver.solve(board) {
                        let mut applied = 0;
                        for action in actions.actions() {
                            match changer.apply(&clone, action) {
                                ChangeResult::None => (),
                                ChangeResult::Valid(after, _) => {
                                    applied += 1;
                                    clone = *after;
                                }
                                ChangeResult::Invalid(_, _, _, errors) => {
                                    println!(
                                        "\n==> Applying {} will cause errors\n    {}\n",
                                        solver.label(),
                                        action
                                    );
                                    errors.print_errors();
                                    return Err(());
                                }
                            }
                        }
                        if applied > 0 {
                            any_applied = true;
                            println!("\n==> Applied {}", pluralize(applied, solver.label()));
                        }
                    }
                    Ok(())
                });

                if any_applied {
                    deductions = None;
                    highlight = None;
                    boards.push(clone);
                    println!();
                    show_board = true;
                } else {
                    println!("\n==> No deductions applied\n");
                }
            }
            "B" => {
                let runtime = Instant::now();
                match find_brute_force(board, false, 0, MAXIMUM_SOLUTIONS) {
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
                    BruteForceResult::Solved(solution) => {
                        println!(
                            "\n==> The puzzle was solved - took {} µs",
                            format_runtime(runtime.elapsed())
                        );
                        boards.push(*solution);
                        println!();
                        show_board = true;
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
                let mut effects = Effects::new();
                for (cell, known) in board.known_iter() {
                    reset.set_given(cell, known, &mut effects);
                }
                if effects.has_errors() {
                    println!("\n==> Invalid board\n");
                    effects.print_errors();
                }
                deductions = None;
                highlight = None;
                boards.push(reset);
                println!();
                show_board = true;
            }
            "Z" => {
                if boards.len() > 1 {
                    println!("\n==> Undoing last move\n");
                    deductions = None;
                    highlight = None;
                    boards.pop();
                    show_board = true;
                }
            }

            "?" => print_help(),
            "Q" => break,

            _ => println!("\n==> Unknown command: {}\n", input[0]),
        }
    }
}

// Used: ABC.EFGH....MNOPQRS..VWX.Z
//
// Want:
// - Y for redo
// - D for deductions?
// - L for lock candidate(s)
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
        "  G <cells> <digit>   - set the given (clue) for a cell\n",
        "  S <cells> <digit>   - solve a cell\n",
        "  E <cells> <digits>  - erase one or more candidates\n",
        "\n",
        "  F [cell | digit]    - find deductions\n",
        "  H <num>             - highlight a single deduction\n",
        "  A [num]             - apply a single or all deductions\n",
        "  V                   - verify that puzzle is solvable\n",
        "  B                   - use Bowman's Bingo to solve the puzzle if possible\n",
        "  R                   - reset candidates based on solved cells\n",
        "  Z                   - undo last change\n",
        "\n",
        "  ?                   - this help message\n",
        "  Q                   - quit\n",
        "\n",
        "      <option> - H, N or I\n",
        "      <cell>   - A1 to J9\n",
        "      <digit>  - 1 to 9\n",
        "      <num>    - any positive number\n",
        "      <char>   - any single character\n",
        "      [...]    - optional\n",
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
        let input = input.trim().replace(' ', "").replace(MISSING, ".");
        if input.is_empty() {
            println!();
            return None;
        }
        if input.to_uppercase() == "E" {
            println!("\n==> Starting an empty puzzle\n");
            return Some(Board::new());
        }

        let parser: Option<Box<dyn Parser>> = if input.len() == 162 {
            Some(Box::new(Parse::wiki()))
        } else if input.len() <= 81 {
            Some(Box::new(Parse::packed_with_player(changer)))
        } else {
            None
        };
        if let Some(parser) = parser {
            let (board, effects, failure) = parser.parse(&input);

            if let Some((cell, known)) = failure {
                println!();
                print_all_and_single_candidates(&board);
                println!("\n==> Setting {} to {} will cause errors\n", cell, known);
                effects.print_errors();
            } else {
                println!();
                print_all_and_single_candidates(&board);
            }

            return Some(board);
        }

        println!(
            concat!(
            "\n==> Expected 81 or 162 digits, got {}\n\n",
            "{}\n",
            "        |        |        |        |        |        |        |        |        |\n",
            ),
            input.len(),
            input
        );
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
