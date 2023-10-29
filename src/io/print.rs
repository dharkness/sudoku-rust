//! Provides functions for printing the state of a puzzle to the console.
//!
//! See <https://www.w3.org/TR/xml-entity-names/025.html>

use crate::layout::{Cell, House, Known};
use crate::puzzle::Board;
use crate::symbols::{CANDIDATE, GIVEN, MISSING};

pub fn print_givens(board: &Board) {
    print_single_value_board(|cell| {
        let value = board.value(cell);
        if value.is_unknown() {
            ' '
        } else if board.is_given(cell) {
            value.label()
        } else {
            MISSING
        }
    });
}

pub fn print_known_values(board: &Board) {
    print_single_value_board(|cell| {
        let value = board.value(cell);
        if value.is_unknown() {
            ' '
        } else {
            value.label()
        }
    });
}

pub fn print_candidate(board: &Board, candidate: Known) {
    print_single_value_board(|cell| {
        if board.is_candidate(cell, candidate) {
            CANDIDATE
        } else {
            let value = board.value(cell);
            if value.is_unknown() {
                ' '
            } else if value == candidate.value() {
                value.label()
            } else if board.is_given(cell) {
                GIVEN
            } else {
                MISSING
            }
        }
    });
}

pub fn print_single_value_board(get_char: impl Fn(Cell) -> char) {
    println!("    ¹ ² ³   ⁴ ⁵ ⁶   ⁷ ⁸ ⁹");
    println!("  ┍───────┬───────┬───────┐");
    House::rows_iter().for_each(|row| {
        if !row.is_top() {
            if row.is_block_top() {
                println!("  ├───────┼───────┼───────┤");
            } else {
                println!("  │       │       │       │");
            }
        }
        print!("{}", row.console_label());
        row.cells().iter().for_each(|cell| {
            let char = get_char(cell);
            if cell.column().is_block_left() {
                print!(" │ {}", char);
            } else {
                print!(" {}", char);
            }
        });
        println!(" │ {}", row.console_label());
    });
    println!("  └───────┴───────┴───────┘");
    println!("    ₁ ₂ ₃   ₄ ₅ ₆   ₇ ₈ ₉");
}

pub fn print_candidates(board: &Board) {
    println!("     ¹   ²   ³     ⁴   ⁵   ⁶     ⁷   ⁸   ⁹");
    println!("  ┍─────────────┬─────────────┬─────────────┐");
    House::rows_iter().for_each(|row| {
        let mut lines = [
            String::from("  │ "),
            format!("{} │ ", row.console_label()),
            String::from("  │ "),
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
                if board.is_given(cell) {
                    lines[2].push_str(&format!(" {} ", MISSING));
                } else {
                    lines[2].push_str("   ");
                }
            }
            if column.is_block_right() {
                lines.iter_mut().for_each(|line| line.push_str(" │ "));
            } else {
                lines.iter_mut().for_each(|line| line.push(' '));
            }
        });
        lines[1].push_str(&format!("{}", row.console_label()));
        lines.iter().for_each(|line| println!("{}", line));
        if row.is_block_bottom() {
            if !row.is_bottom() {
                println!("  ├─────────────┼─────────────┼─────────────┤");
            }
        } else {
            println!("  │             │             │             │");
        }
    });
    println!("  └─────────────┴─────────────┴─────────────┘");
    println!("     ₁   ₂   ₃     ₄   ₅   ₆     ₇   ₈   ₉");
}
