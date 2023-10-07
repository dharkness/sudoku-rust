//! Provides various strategies for validating and solving Sudoku puzzles.

mod algorithms;
mod deadly_rectangles;
mod reporter;
mod solver;
mod technique;

pub use algorithms::{find_brute_force, find_intersection_removals, BruteForceResult};
pub use deadly_rectangles::creates_deadly_rectangles;
pub use reporter::Reporter;
pub use solver::{Resolution, Solver};
pub use technique::{Difficulty, MANUAL_TECHNIQUES, NON_PEER_TECHNIQUES};
