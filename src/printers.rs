use crate::layout::{House, Known};
use crate::puzzle::Board;

const MISSING: char = '·';
const ROW_COORDS: [char; 9] = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J'];
const VALUES: [char; 10] = [MISSING, '1', '2', '3', '4', '5', '6', '7', '8', '9'];

pub fn print_values(board: &Board) {
    println!("  ¹²³⁴⁵⁶⁷⁸⁹");
    for (r, coord) in ROW_COORDS.iter().enumerate() {
        let row = House::row(r.into());
        print!("{} ", coord);
        for c in 0..9 {
            let value = board.value(row.cell(c.into()));
            if value == Known::UNKNOWN {
                print!("{}", MISSING);
            } else {
                print!("{}", value);
            }
        }
        println!();
    }
}

pub fn print_candidates(board: &Board) {
    println!("   ¹   ²   ³     ⁴   ⁵   ⁶     ⁷   ⁸   ⁹");
    for (r, coord) in ROW_COORDS.iter().enumerate() {
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
            if value == Known::UNKNOWN {
                for k in Known::ALL {
                    let line = k.usize() / 3;
                    if candidates[k] {
                        lines[line].push(VALUES[k.value() as usize]);
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
            if c % 3 == 2 {
                if c < 8 {
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
        println!("{} {}", lines[1], ROW_COORDS[r]);
        println!("{}", lines[2]);
        if r % 3 == 2 {
            if r < 8 {
                println!("  ------------+-------------+------------");
            }
        } else {
            println!("              |             |");
        }
    }
    println!("   ₁   ₂   ₃     ₄   ₅   ₆     ₇   ₈   ₉");
}
