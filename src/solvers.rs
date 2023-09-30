//! Provides various strategies for validating and solving Sudoku puzzles.

use itertools::Itertools;

pub mod deadly_rectangles;

pub mod avoidable_rectangles;
pub mod bugs;
pub mod empty_rectangles;
pub mod fish;
pub mod hidden_tuples;
pub mod intersection_removals;
pub mod naked_tuples;
pub mod singles_chains;
pub mod skyscrapers;
pub mod unique_rectangles;
pub mod xy_chains;
pub mod xyz_wings;
pub mod y_wings;

use crate::layout::*;
use crate::puzzle::*;

pub type Solver = fn(&Board) -> Option<Effects>;

pub const SOLVERS: [Solver; 19] = [
    intersection_removals::find_intersection_removals,
    naked_tuples::find_naked_pairs,
    naked_tuples::find_naked_triples,
    naked_tuples::find_naked_quads,
    hidden_tuples::find_hidden_pairs,
    hidden_tuples::find_hidden_triples,
    hidden_tuples::find_hidden_quads,
    fish::find_x_wings,
    fish::find_swordfish,
    fish::find_jellyfish,
    singles_chains::find_singles_chains,
    skyscrapers::find_skyscrapers,
    y_wings::find_y_wings,
    xyz_wings::find_xyz_wings,
    avoidable_rectangles::find_avoidable_rectangles,
    xy_chains::find_xy_chains,
    unique_rectangles::find_unique_rectangles,
    empty_rectangles::find_empty_rectangles,
    bugs::find_bugs,
];

pub const SOLVER_LABELS: [&str; 19] = [
    "intersection removal",
    "naked pair",
    "naked triple",
    "naked quad",
    "hidden pair",
    "hidden triple",
    "hidden quad",
    "x-wing",
    "swordfish",
    "jellyfish",
    "singles chain",
    "skyscraper",
    "y-wing",
    "xyz-wing",
    "avoidable rectangle",
    "xy-chain",
    "unique rectangle",
    "empty rectangle",
    "bug",
];
