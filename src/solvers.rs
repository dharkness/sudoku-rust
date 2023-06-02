//! Provides various strategies for validating and solving Sudoku puzzles.

use crate::puzzle::{Board, Effects};

pub mod deadly_rectangles;

mod distinct_tuples;
pub mod hidden_tuples;
pub mod intersection_removals;
pub mod naked_tuples;

pub type Solver = fn (&Board) -> Option<Effects>;
