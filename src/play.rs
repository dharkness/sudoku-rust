use std::io::{stdout, Write};

use crate::effects::Effects;
use crate::generate::Generator;
use crate::layout::{Board, Cell, Known};
use crate::printers::print_candidates;

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
                    boards.push(board)
                }
                println!();
                show = true
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
                println!();
                show = true
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
                if !effects.apply_all(&mut clone) {
                    println!("\n==> Invalid move\n");
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
                if !effects.apply_all(&mut clone) {
                    println!("\n==> Invalid move\n");
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
        "N                - start a new puzzle\n",
        "G                - generate a random puzzle\n",
        "P                - print the puzzle\n",
        "E <cell> <value> - erase candidate\n",
        "S <cell> <value> - solve cell\n",
        "Z                - undo last move\n",
        "H                - this help message\n",
        "Q                - quit\n\n",
        "Note: commands and cells are not case-sensitive\n"
    ))
}

fn create_new_puzzle() -> Option<Board> {
    println!(concat!(
        "\n==> Enter the givens\n\n",
        "- enter up to 81 digits or periods\n",
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
                "\n==> Expected 81 digits, got {} - |{}|\n",
                input.len(),
                input
            );
            continue;
        }

        let mut board = Board::new();
        for (i, char) in input.chars().enumerate() {
            if char == '.' {
                continue;
            }

            if !('1'..='9').contains(&char) {
                println!("\n==> Expected digit or period, got {}\n", char);
                break 'input;
            }

            let cell = Cell::from(i);
            let known = Known::from(char);
            if !board.is_candidate(cell, known) {
                println!("\n==> {} is not a candidate for {}\n", known, cell);
                break 'input;
            }

            let mut effects = Effects::new();
            board.set_known(cell, known, &mut effects);
            if !effects.apply_all(&mut board) {
                println!("\n==> Invalid puzzle after setting {} to {}\n", cell, known);
                break 'input;
            }
        }

        return Some(board);
    }

    None
}
