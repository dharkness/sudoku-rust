//! Provides a text-based interface for creating and playing Sudoku puzzles.

use std::io::{stdout, Write};

use crate::layout::{Cell, Known};
use crate::printers::{print_candidate, print_candidates};
use crate::puzzle::{Board, Effects, Generator, Parser};
use crate::solvers::Solver;
use crate::symbols::UNKNOWN_VALUE;

const URL: &str = "https://www.sudokuwiki.org/sudoku.htm?bd=";

const SOLVERS: [Solver; 12] = [
    crate::solvers::intersection_removals::find_intersection_removals,
    crate::solvers::naked_tuples::find_naked_pairs,
    crate::solvers::naked_tuples::find_naked_triples,
    crate::solvers::naked_tuples::find_naked_quads,
    crate::solvers::hidden_tuples::find_hidden_pairs,
    crate::solvers::hidden_tuples::find_hidden_triples,
    crate::solvers::hidden_tuples::find_hidden_quads,
    crate::solvers::fish::find_x_wings,
    crate::solvers::fish::find_swordfish,
    crate::solvers::fish::find_jellyfish,
    crate::solvers::singles_chains::find_singles_chains,
    crate::solvers::y_wings::find_y_wings,
];
const SOLVER_LABELS: [&str; 12] = [
    "intersection removal",
    "naked pair",
    "naked triple",
    "naked quad",
    "hidden pair",
    "hidden triple",
    "hidden quad",
    "x-wing",
    "swordfish",
    "jellyfish",
    "singles chain",
    "y-wing",
];

pub fn play() {
    let mut boards = vec![Board::new()];
    let mut show = false;

    print_help();
    loop {
        let board = boards.last().unwrap();
        if show {
            print_candidates(board);
            println!();
            show = false;
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
            "N" => {
                if let Some(board) = create_new_puzzle() {
                    boards.push(board);
                    println!();
                    show = true
                }
            }
            "G" => {
                println!();
                let mut generator = Generator::new();
                match generator.generate() {
                    Some(board) => {
                        println!("\n==> Clues: {}\n", board);
                        boards.push(board);
                    }
                    None => {
                        println!("\n==> Failed to generate a puzzle\n");
                    }
                }
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
                    show = true
                }
            }
            "X" => {
                let mut unknown = UNKNOWN_VALUE;
                if input.len() >= 2 {
                    unknown = input[1].chars().next().unwrap_or(unknown);
                }
                println!("\n==> {}\n", board.packed_string(unknown));
            }
            "W" => {
                println!("\n==> {}{}\n", URL, board.url_string().replace(' ', ""));
            }
            "E" => {
                if input.len() != 3 {
                    println!("\n==> c <cell> <value>\n");
                    continue;
                }
                let cell = Cell::from(input[1]);
                let known = Known::from(input[2]);
                if !board.is_candidate(cell, known) {
                    println!("\n==> {} is not a candidate for {}\n", known, cell);
                    continue;
                }
                let mut clone = *board;
                let mut effects = Effects::new();
                clone.remove_candidate(cell, known, &mut effects);
                if let Some(errors) = effects.apply_all(&mut clone) {
                    println!("\n==> Invalid move\n");
                    errors.print_errors();
                    println!();
                    continue;
                }
                boards.push(clone);
                if clone.is_solved() {
                    println!("\n==> Congratulations!\n");
                } else {
                    println!();
                    show = true;
                }
            }
            "S" => {
                if input.len() != 3 {
                    println!("\n==> s <cell> <value>\n");
                    continue;
                }
                let cell = Cell::from(input[1].to_uppercase());
                let known = Known::from(input[2]);
                if !board.is_candidate(cell, known) {
                    println!("\n==> {} is not a candidate for {}\n", known, cell);
                    continue;
                }
                let mut clone = *board;
                let mut effects = Effects::new();
                clone.set_known(cell, known, &mut effects);
                if let Some(errors) = effects.apply_all(&mut clone) {
                    println!("\n==> Invalid move\n");
                    errors.print_errors();
                    println!();
                    continue;
                }
                boards.push(clone);
                if clone.is_solved() {
                    println!("\n==> Congratulations!\n");
                } else {
                    println!();
                    show = true;
                }
            }
            "F" => {
                let mut found = false;
                SOLVERS.iter().enumerate().for_each(|(i, solver)| {
                    if let Some(effects) = solver(board) {
                        found = true;
                        println!(
                            "\n==> Found {}\n",
                            pluralize(effects.action_count(), SOLVER_LABELS[i])
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
                let mut found = false;
                let mut clone = *board;
                let _ = SOLVERS.iter().enumerate().try_for_each(|(i, solver)| {
                    if let Some(effects) = solver(board) {
                        found = true;
                        if let Some(errors) = effects.apply_all(&mut clone) {
                            println!(
                                "\n==> Found errors while applying {}\n",
                                pluralize(effects.action_count(), SOLVER_LABELS[i])
                            );
                            errors.print_errors();
                            println!();
                            return Err(());
                        }
                        println!(
                            "\n==> Applied {}",
                            pluralize(effects.action_count(), SOLVER_LABELS[i])
                        );
                    }
                    Ok(())
                });

                if found {
                    boards.push(clone);
                    if clone.is_solved() {
                        println!("\n==> Congratulations!\n");
                    } else {
                        println!();
                        show = true;
                    }
                } else {
                    println!("\n==> No deductions found\n");
                }
            }
            "R" => {
                let mut reset = Board::new();
                board.known_iter().for_each(|(cell, known)| {
                    let mut effects = Effects::new();
                    reset.set_given(cell, known, &mut effects);
                    if let Some(errors) = effects.apply_all(&mut reset) {
                        println!("\n==> Invalid board\n");
                        errors.print_errors();
                        println!();
                    }
                });
                boards.push(reset);
                println!();
                show = true;
            }
            "Z" => {
                if boards.len() > 1 {
                    println!("\n==> Undoing last move\n");
                    boards.pop();
                    show = true
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
        "\n==> Help\n\n",
        "N                - start or input a new puzzle\n",
        "G                - generate a random puzzle\n",
        "P <value>        - print the puzzle, optionally limited to a single candidate\n",
        "X [char]         - export the puzzle with optional character for unsolved cells\n",
        "W                - print URL to play on SudokuWiki.org\n",
        "E <cell> <value> - erase a candidate\n",
        "S <cell> <value> - solve a cell\n",
        "F                - find deductions\n",
        "A                - apply deductions\n",
        "R                - reset candidates based on solved cells\n",
        "Z                - undo last change\n",
        "H                - this help message\n",
        "Q                - quit\n\n",
        "Note: commands and cells are not case-sensitive\n"
    ))
}

fn create_new_puzzle() -> Option<Board> {
    println!(concat!(
        "\n==> Enter the givens\n\n",
        "- enter up to 81 digits\n",
        "- use period or zero to leave a cell blank\n",
        "- spaces are ignored\n",
        "- leave empty to cancel\n",
        "- enter 'E' for an empty puzzle\n",
    ));

    'input: loop {
        print!("> ");
        let _ = stdout().flush();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().replace(' ', "").to_uppercase();
        if input.is_empty() {
            return None;
        }
        if input == "E" {
            return Some(Board::new());
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

        let parser = Parser::new(true, false);
        let (board, effects, failure) = parser.parse(&input);

        if let Some((cell, known)) = failure {
            println!("\n==> Setting {} to {} caused errors\n", cell, known);
            effects.print_errors();
            println!();
            print_candidates(&board);
            println!();
            break 'input;
        }

        return Some(board);
    }

    None
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
