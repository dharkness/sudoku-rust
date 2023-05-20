use super::board::Board;
use super::cell_set::Cell;
use crate::known_set;
use crate::known_set::{ALL_KNOWNS, UNKNOWN};

const MISSING: char = '·';
const ROW_COORDS: [char; 9] = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J'];
const VALUES: [char; 10] = [MISSING, '1', '2', '3', '4', '5', '6', '7', '8', '9'];

pub fn print_values(board: &Board) {
    println!("  123456789");
    for (row, coord) in ROW_COORDS.iter().enumerate() {
        print!("{} ", coord);
        for col in 0..9 {
            let value = board.value((9 * row + col) as Cell);
            if value == UNKNOWN {
                print!("{}", MISSING);
            } else {
                print!("{}", value);
            }
        }
        println!();
    }
}

pub fn print_candidates(board: &Board) {
    println!("   1   2   3     4   5   6     7   8   9\n");
    for (row, coord) in ROW_COORDS.iter().enumerate() {
        let mut lines = [
            String::from("  "),
            coord.to_string() + " ",
            String::from("  "),
        ];
        for col in 0..9 {
            let cell = (9 * row + col) as Cell;
            let value = board.value(cell);
            let candidates = board.candidates(cell);
            if value == UNKNOWN {
                for k in ALL_KNOWNS {
                    let line = ((k - 1) / 3) as usize;
                    if known_set::has(candidates, k) {
                        lines[line].push(VALUES[k as usize]);
                    } else {
                        lines[line].push('·');
                    }
                }
            } else {
                // for line in lines.iter_mut().take(3) {
                //     line.push(VALUES[value as usize]);
                //     line.push(VALUES[value as usize]);
                //     line.push(VALUES[value as usize]);
                // }
                lines[0].push_str("   ");
                lines[1].push_str(&format!(" {} ", VALUES[value as usize]));
                lines[2].push_str("   ");
            }
            if col % 3 == 2 {
                if col < 8 {
                    for line in lines.iter_mut().take(3) {
                        line.push_str(" | ");
                    }
                }
            } else {
                for line in lines.iter_mut().take(3) {
                    line.push(' ');
                }
            }
        }
        println!("{}", lines[0]);
        println!("{} {}", lines[1], ROW_COORDS[row]);
        println!("{}", lines[2]);
        if row % 3 == 2 {
            if row < 8 {
                println!("  ------------+-------------+------------");
            }
        } else {
            println!("              |             |");
        }
    }
    println!("\n   1   2   3     4   5   6     7   8   9");
}
