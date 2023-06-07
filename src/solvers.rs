//! Provides various strategies for validating and solving Sudoku puzzles.

use itertools::Itertools;

pub mod deadly_rectangles;

pub mod fish;
pub mod hidden_tuples;
pub mod intersection_removals;
pub mod naked_tuples;
pub mod singles_chains;

use crate::layout::*;
use crate::puzzle::*;

pub type Solver = fn(&Board) -> Option<Effects>;
