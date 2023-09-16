//! Provides functions for printing the state of a puzzle to the console.

use crate::layout::{Cell, House, Known};
use crate::puzzle::Board;
use crate::symbols::{MISSING, REMOVE_CANDIDATE};

pub fn print_values(board: &Board) {
    println!("  ¹²³⁴⁵⁶⁷⁸⁹");
    House::rows_iter().for_each(|row| {
        print!("{} ", row.console_label());
        row.cells().iter().for_each(|cell| {
            let value = board.value(cell);
            print!("{}", value);
        });
        println!();
    });
}

pub fn print_candidate(board: &Board, candidate: Known) {
    println!("  ¹²³⁴⁵⁶⁷⁸⁹");
    House::rows_iter().for_each(|row| {
        print!("{} ", row.console_label());
        row.cells().iter().for_each(|cell| {
            if board.is_candidate(cell, candidate) {
                print!("{}", candidate.highlight());
            } else {
                let value = board.value(cell);
                if value.is_unknown() || value == candidate.value() {
                    print!("{}", value);
                } else {
                    print!("{}", REMOVE_CANDIDATE);
                }
            }
        });
        println!();
    });
}

pub fn print_candidates(board: &Board) {
    println!("   ¹   ²   ³     ⁴   ⁵   ⁶     ⁷   ⁸   ⁹");
    House::rows_iter().for_each(|row| {
        let mut lines = [
            String::from("  "),
            format!("{} ", row.console_label()),
            String::from("  "),
        ];
        House::columns_iter().for_each(|column| {
            let cell = Cell::from_row_column(row, column);
            let value = board.value(cell);
            let candidates = board.candidates(cell);
            if !value {
                for k in Known::iter() {
                    let line = k.usize() / 3;
                    if candidates[k] {
                        lines[line].push(k.label());
                    } else {
                        lines[line].push(MISSING);
                    }
                }
            } else {
                lines[0].push_str("   ");
                lines[1].push_str(&format!(" {} ", value));
                lines[2].push_str("   ");
            }
            if column.is_block_right() {
                if !column.is_right() {
                    lines.iter_mut().for_each(|line| line.push_str(" │ "));
                }
            } else {
                lines.iter_mut().for_each(|line| line.push(' '));
            }
        });
        lines[1].push_str(&format!(" {}", row.console_label()));
        lines.iter().for_each(|line| println!("{}", line));
        if row.is_block_bottom() {
            if !row.is_bottom() {
                println!("  ────────────┼─────────────┼────────────");
            }
        } else {
            println!("              │             │");
        }
    });
    println!("   ₁   ₂   ₃     ₄   ₅   ₆     ₇   ₈   ₉");
}
