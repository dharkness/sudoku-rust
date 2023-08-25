//! Provides various strategies for validating and solving Sudoku puzzles.

use itertools::Itertools;

pub mod deadly_rectangles;

pub mod bugs;
pub mod empty_rectangles;
pub mod fish;
pub mod hidden_tuples;
pub mod intersection_removals;
pub mod naked_tuples;
pub mod singles_chains;
pub mod skyscrapers;
pub mod unique_rectangles;
pub mod xyz_wings;
pub mod y_wings;

use crate::layout::*;
use crate::puzzle::*;

pub type Solver = fn(&Board) -> Option<Effects>;
