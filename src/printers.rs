//! Provides functions for printing the state of a puzzle to the console.

use crate::layout::{House, Known};
use crate::puzzle::Board;
use crate::symbols::{MISSING, ROW_COORDS};

pub fn print_values(board: &Board) {
    println!("  ¹²³⁴⁵⁶⁷⁸⁹");
    House::all_rows().iter().for_each(|row| {
        print!("{} ", row.console_label());
        row.cells().iter().for_each(|cell| {
            let value = board.value(cell);
            print!("{}", value);
        });
        println!();
    });
}

pub fn print_candidates(board: &Board) {
    println!("   ¹   ²   ³     ⁴   ⁵   ⁶     ⁷   ⁸   ⁹");
    ROW_COORDS.iter().enumerate().for_each(|(r, coord)| {
        let row = House::row(r.into());
        let mut lines = [
            String::from("  "),
            coord.to_string() + " ",
            String::from("  "),
        ];
        for c in 0..9 {
            let cell = row.cell(c.into());
            let value = board.value(cell);
            let candidates = board.candidates(cell);
            if !value {
                for k in Known::ALL {
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
            if c % 3 == 2 {
                if c < 8 {
                    lines.iter_mut().for_each(|line| line.push_str(" | "));
                }
            } else {
                lines.iter_mut().for_each(|line| line.push(' '));
            }
        }
        println!("{}", lines[0]);
        println!("{} {}", lines[1], ROW_COORDS[r]);
        println!("{}", lines[2]);
        if r % 3 == 2 {
            if r < 8 {
                println!("  ------------+-------------+------------");
            }
        } else {
            println!("              |             |");
        }
    });
    println!("   ₁   ₂   ₃     ₄   ₅   ₆     ₇   ₈   ₉");
}
