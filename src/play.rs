use std::io::{stdout, Write};
use crate::effects::Effects;

use crate::layout::{Board, Cell, Known};
use crate::printers::print_candidates;

pub fn play_puzzle() {
    let mut boards = vec![Board::new()];

    loop {
        let board = boards.last().unwrap();
        print_candidates(&board);
        println!();

        print!("> ");
        let _ = stdout().flush();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().split(' ').collect::<Vec<_>>();

        match input[0] {
            "e" => {
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
                let mut clone = board.clone();
                let mut effects = Effects::new();
                clone.remove_candidate(cell, known, &mut effects);
                if !apply_all_actions(&mut clone, &effects) {
                    continue;
                }
                boards.push(clone);
            },
            "s" => {
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
                let mut clone = board.clone();
                let mut effects = Effects::new();
                clone.set_known(cell, known, &mut effects);
                if !apply_all_actions(&mut clone, &effects) {
                    continue;
                }
                boards.push(clone);
            },
            "z" => {
                if boards.len() > 1 {
                    println!("\n==> Undoing last move\n");
                    boards.pop();
                }
            },
            "q" => break,
            _ => continue,
        }
    }
}

fn apply_all_actions(board: &mut Board, effects: &Effects) -> bool {
    let mut effects = effects.clone();
    while effects.has_actions() {
        println!("\n{:?}\n", effects);
        let mut next = Effects::new();
        effects.apply(board, &mut next);
        if next.has_errors() {
            println!("\n==> Invalid move\n");
            print_candidates(board);
            return false;
        }
        effects = next;
    }
    true
}