//! Provides various strategies for validating and solving Sudoku puzzles.

mod algorithms;
mod deadly_rectangles;
mod solver;
mod technique;

pub use algorithms::find_intersection_removals;
pub use deadly_rectangles::creates_deadly_rectangles;
pub use solver::{Resolution, Solver};
pub use technique::{Difficulty, TECHNIQUES};
