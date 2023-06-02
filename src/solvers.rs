//! Provides various strategies for validating and solving Sudoku puzzles.

pub mod deadly_rectangles;

pub mod hidden_tuples;
pub mod intersection_removals;
pub mod naked_tuples;

mod distinct_tuples;

use crate::layout::{Cell, CellSet, Coord, House, Known, KnownSet, Rectangle};
use crate::puzzle::{Action, Board, Effects, Strategy};
use distinct_tuples::*;

pub type Solver = fn(&Board) -> Option<Effects>;
