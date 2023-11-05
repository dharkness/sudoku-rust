//! Provides functions for printing the state of a puzzle to the console.
//!
//! See <https://www.w3.org/TR/xml-entity-names/025.html>

use crate::layout::{Cell, House, Known};
use crate::puzzle::Board;
use crate::symbols::{GIVEN, MISSING};

// Unicode line characters: https://www.w3.org/TR/xml-entity-names/025.html
//
// thin:   ┌ ─ ┐ └ ┘ ├ ┤ ┬ ┴ ┼
//
// think:  ┏ ━ ┓ ┗ ┛ ┣ ┫ ┳ ┻ ╋
// combo:  ┠ ┨ ┯ ┷ ┿ ╂
//
// double: ╔ ═ ╗ ╚ ╝ ╠ ╣ ╦ ╩ ╬
// combo:  ╟ ╢ ╧ ╤ ╪ ╫
//
// dashed: ┄ ┅ ┆ ┇ ┈ ┉ ┊ ┋ ╌ ╍ ╎ ╏

pub fn print_givens(board: &Board) {
    for line in add_single_value_labels(write_givens(board)) {
        println!("{}", line);
    }
}

pub fn write_givens(board: &Board) -> Vec<String> {
    write_single_value(|cell| {
        let value = board.value(cell);
        if value.is_unknown() {
            ' '
        } else if board.is_given(cell) {
            value.label()
        } else {
            MISSING
        }
    })
}

pub fn print_known_values(board: &Board) {
    for line in add_single_value_labels(write_known_values(board)) {
        println!("{}", line);
    }
}

pub fn write_known_values(board: &Board) -> Vec<String> {
    write_single_value(|cell| {
        let value = board.value(cell);
        if value.is_unknown() {
            ' '
        } else {
            value.label()
        }
    })
}

pub fn print_candidate(board: &Board, candidate: Known) {
    for line in add_single_value_labels(write_candidate(board, candidate)) {
        println!("{}", line);
    }
}

pub fn write_candidate(board: &Board, candidate: Known) -> Vec<String> {
    write_single_value(|cell| {
        if board.is_candidate(cell, candidate) {
            GIVEN
        } else {
            let value = board.value(cell);
            if value.is_unknown() {
                ' '
            } else if value == candidate.value() {
                value.label()
            } else {
                ' '
            }
        }
    })
}

pub fn add_single_value_labels(grid: Vec<String>) -> Vec<String> {
    let mut lines = Vec::new();
    let mut iter = grid.into_iter();

    lines.push("    1 2 3   4 5 6   7 8 9    ".to_owned());
    lines.push(format!("  {}  ", iter.next().unwrap()));
    for row in House::rows_iter() {
        lines.push(format!(
            "{} {} {}",
            row.console_label(),
            iter.next().unwrap(),
            row.console_label()
        ));
        if row.is_block_bottom() {
            lines.push(format!("  {}  ", iter.next().unwrap()));
        }
    }
    lines.push("    1 2 3   4 5 6   7 8 9    ".to_owned());

    lines
}

pub fn write_single_value(get_char: impl Fn(Cell) -> char) -> Vec<String> {
    let mut lines = Vec::new();

    lines.push("┍───────┬───────┬───────┐".to_owned());
    House::rows_iter().for_each(|row| {
        if !row.is_top() {
            if row.is_block_top() {
                lines.push("├───────┼───────┼───────┤".to_owned());
            } else {
                // lines.push("│       │       │       │");
            }
        }
        let mut line = String::from('│');
        row.cells().iter().for_each(|cell| {
            line.push(' ');
            line.push(get_char(cell));
            if cell.column().is_block_right() {
                line.push_str(" │");
            }
        });
        lines.push(line);
    });
    lines.push("└───────┴───────┴───────┘".to_owned());

    lines
}

pub fn print_candidates(board: &Board) {
    for line in write_candidates_with_labels(board) {
        println!("{}", line);
    }
}

pub fn write_candidates_with_labels(board: &Board) -> Vec<String> {
    let mut lines = Vec::new();
    let mut grid = write_candidates(board).into_iter();

    lines.push("     1    2    3     4    5    6     7    8    9     ".to_owned());
    lines.push(format!("  {}  ", grid.next().unwrap()));
    for row in House::rows_iter() {
        lines.push(format!("  {}  ", grid.next().unwrap()));
        lines.push(format!(
            "{} {} {}",
            row.console_label(),
            grid.next().unwrap(),
            row.console_label()
        ));
        lines.push(format!("  {}  ", grid.next().unwrap()));
        lines.push(format!("  {}  ", grid.next().unwrap()));
    }
    lines.push("     1    2    3     4    5    6     7    8    9     ".to_owned());

    lines
}

pub fn write_candidates(board: &Board) -> Vec<String> {
    let mut lines = Vec::new();

    lines.push("┍───────────────┬───────────────┬───────────────┐".to_owned());
    House::rows_iter().for_each(|row| {
        let mut cell_lines = [String::from("│ "), String::from("│ "), String::from("│ ")];
        House::columns_iter().for_each(|column| {
            let cell = Cell::from_row_column(row, column);
            let value = board.value(cell);
            let candidates = board.candidates(cell);
            if !value {
                for k in Known::iter() {
                    let line = k.usize() / 3;
                    if candidates[k] {
                        cell_lines[line].push(k.label());
                    } else {
                        cell_lines[line].push(MISSING);
                    }
                }
            } else {
                cell_lines[0].push_str("   ");
                cell_lines[1].push_str(&format!(" {} ", value));
                if board.is_given(cell) {
                    cell_lines[2].push_str(&format!(" {} ", MISSING));
                } else {
                    cell_lines[2].push_str("   ");
                }
            }
            if column.is_right() {
                cell_lines.iter_mut().for_each(|line| line.push_str(" │"));
            } else if column.is_block_right() {
                cell_lines.iter_mut().for_each(|line| line.push_str(" │ "));
            } else {
                cell_lines.iter_mut().for_each(|line| line.push_str("  "));
            }
        });
        cell_lines.into_iter().for_each(|line| lines.push(line));
        if row.is_block_bottom() {
            if !row.is_bottom() {
                lines.push("├───────────────┼───────────────┼───────────────┤".to_owned());
            }
        } else {
            lines.push("│               │               │               │".to_owned());
        }
    });
    lines.push("└───────────────┴───────────────┴───────────────┘".to_owned());

    lines
}

pub fn print_all_and_single_candidates(board: &Board) {
    let mut columns = [Vec::new(), Vec::new(), Vec::new()];

    for (i, grid) in Known::iter().map(|k| write_candidate(board, k)).enumerate() {
        columns[i % 3].extend(grid);
    }

    let mut columns_iter = columns.into_iter();
    let mut column_iters = [
        columns_iter.next().unwrap().into_iter(),
        columns_iter.next().unwrap().into_iter(),
        columns_iter.next().unwrap().into_iter(),
    ];
    for line in write_candidates_with_labels(board) {
        println!(
            "{}    {} {} {}",
            line,
            column_iters[0].next().unwrap(),
            column_iters[1].next().unwrap(),
            column_iters[2].next().unwrap()
        );
    }
}
