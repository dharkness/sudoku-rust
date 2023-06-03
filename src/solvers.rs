//! Provides various strategies for validating and solving Sudoku puzzles.

use itertools::Itertools;

pub mod deadly_rectangles;

pub mod fish;
pub mod hidden_tuples;
pub mod intersection_removals;
pub mod naked_tuples;

mod distinct_tuples;

use crate::layout::*;
use crate::puzzle::*;

use distinct_tuples::*;

pub type Solver = fn(&Board) -> Option<Effects>;
