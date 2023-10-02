//! Provides a text-based interface for creating and playing Sudoku puzzles.

use clap::Args;
use std::io::{stdout, Write};

use crate::build::Generator;
use crate::io::{
    format_for_fancy_console, format_for_wiki, format_grid, format_packed, print_candidate,
    print_candidates, Cancelable, Parse, SUDOKUWIKI_URL,
};
use crate::layout::{Cell, Known};
use crate::puzzle::{Board, Effects};
use crate::solve::NON_PEER_TECHNIQUES;
use crate::symbols::UNKNOWN_VALUE;

#[derive(Debug, Args)]
pub struct PlayArgs {
    /// Clues for a starting puzzle
    #[clap(long, short = 'p')]
    puzzle: Option<String>,
}

pub fn start_player(args: PlayArgs, canceler: &Cancelable) {
    let mut boards = vec![];
    let mut show_board = false;

    match args.puzzle {
        Some(clues) => {
            let parser = Parse::packed().stop_on_error().remove_peers();
            let (board, effects, failure) = parser.parse(&clues);

            boards.push(board);
            if let Some((cell, known)) = failure {
                println!();
                print_candidates(&board);
                println!();
                effects.print_errors();
                println!("\n==> Setting {} to {} caused errors\n", cell, known);
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
            "N" => {
                if let Some(board) = create_new_puzzle() {
                    boards.push(board);
                    println!();
                    show_board = true
                }
            }
            "G" => {
                println!();
                let mut generator = Generator::new(false);
                match generator.generate(canceler) {
                    Some(board) => {
                        println!("\n==> Clues: {}\n", board);
                        boards.push(board);
                    }
                    None => {
                        println!("\n==> Failed to generate a puzzle\n");
                    }
                }
                canceler.clear();
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
                println!();
                show_board = true;
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
                if effects.has_errors() {
                    println!("\n==> Invalid move\n");
                    effects.print_errors();
                    println!();
                    continue;
                }
                if let Some(errors) = effects.apply_all(&mut clone) {
                    println!("\n==> Invalid move\n");
                    errors.print_errors();
                    println!();
                    continue;
                }
                boards.push(clone);
                println!();
                show_board = true;
            }
            "F" => {
                let mut found = false;
                NON_PEER_TECHNIQUES.iter().for_each(|solver| {
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
                let mut found = false;
                let mut clone = *board;
                let _ = NON_PEER_TECHNIQUES.iter().try_for_each(|solver| {
                    if let Some(effects) = solver.solve(board) {
                        found = true;
                        if let Some(errors) = effects.apply_all(&mut clone) {
                            println!(
                                "\n==> Found errors while applying {}\n",
                                pluralize(effects.action_count(), solver.name())
                            );
                            errors.print_errors();
                            println!();
                            return Err(());
                        }
                        println!(
                            "\n==> Applied {}",
                            pluralize(effects.action_count(), solver.name())
                        );
                    }
                    Ok(())
                });

                if found {
                    boards.push(clone);
                    println!();
                    show_board = true;
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
        "\n==> Help\n\n",
        "  N                - start or input a new puzzle\n",
        "  G                - generate a random puzzle\n",
        "  P <digit>        - print the puzzle, optionally limited to a single candidate\n",
        "  X [char]         - export the puzzle with optional character for unsolved cells\n",
        "  W                - print URL to play on SudokuWiki.org\n",
        "  M                - print the puzzle as a grid suitable for email\n",
        "  E <cell> <digit> - erase a candidate\n",
        "  S <cell> <digit> - solve a cell\n",
        "  F                - find deductions\n",
        "  A                - apply deductions\n",
        "  R                - reset candidates based on solved cells\n",
        "  Z                - undo last change\n",
        "  H                - this help message\n",
        "  Q                - quit\n\n",
        "      <cell>  - A1 to J9\n",
        "      <digit> - 1 to 9\n",
        "      <char>  - any single character\n\n",
        "  Note: commands and cells are not case-sensitive\n",
    ))
}

fn create_new_puzzle() -> Option<Board> {
    println!(concat!(
        "\n==> Enter the givens\n\n",
        "  - enter up to 81 digits\n",
        "  - use period or zero to leave a cell blank\n",
        "  - spaces are ignored\n",
        "  - leave empty to cancel\n",
        "  - enter 'E' for an empty puzzle\n",
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

        let parser = Parse::packed().stop_on_error().remove_peers();
        let (board, effects, failure) = parser.parse(&input);

        if let Some((cell, known)) = failure {
            println!("\n==> Setting {} to {} caused errors\n", cell, known);
            effects.print_errors();
            println!();
            print_candidates(&board);
            println!();
            break 'input;
        } else if effects.has_errors() {
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
