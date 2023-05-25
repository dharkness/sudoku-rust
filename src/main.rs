mod generate;
mod layout;
mod printers;
mod solvers;

use crate::generate::generate_board;
use crate::layout::{Board, Cell, Known};
use crate::printers::{print_candidates, print_values};
use crate::solvers::intersection_removals::find_intersection_removals;

fn main() {
    // let mut board = Board::new();
    // board.remove_candidate(Cell::from("B1"), Known::from("1"));
    // board.remove_candidate(Cell::from("B2"), Known::from("1"));
    // board.remove_candidate(Cell::from("B3"), Known::from("1"));
    // board.remove_candidate(Cell::from("C1"), Known::from("1"));
    // board.remove_candidate(Cell::from("C2"), Known::from("1"));
    // board.remove_candidate(Cell::from("C3"), Known::from("1"));
    // print_candidates(&board);
    // let remove = find_intersection_removals(&board);
    // println!("{:?}", remove)

    generate_board();
}
