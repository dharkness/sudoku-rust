//! Strategy implementations and the solver contract.
//!
//! This module collects all `find_*` routines that inspect a `Board` and
//! return deductions as `Effects`. Each routine must be pure: no board
//! mutation, no I/O, and deterministic results for the same input.
//!
//! Contract:
//! - Signature: `fn find_xxx(board: &Board, single: bool) -> Option<Effects>`
//! - Return `Some(effects)` only when at least one action is present.
//! - Respect `single`: if an action is added and `single` is true, return early.
//! - Use `Action` with the correct `Strategy` label and attach clue metadata.
//!
//! The solver orchestrates ordering and application. Keep algorithms focused,
//! fast, and allocation-light; prefer `CellSet` and `DigitSet` algebra over
//! temporary collections.

use itertools::Itertools;

pub use avoidable_rectangles::find_avoidable_rectangles;
pub use brute_force::{find_brute_force, BruteForceResult};
pub use bugs::find_bugs;
pub use extended_unique_rectangles::find_extended_unique_rectangles;
pub use fireworks::find_fireworks;
pub use fish::find_jellyfish;
pub use fish::find_swordfish;
pub use fish::find_x_wings;
pub use hidden_singles::find_hidden_singles;
pub use hidden_tuples::find_hidden_pairs;
pub use hidden_tuples::find_hidden_quads;
pub use hidden_tuples::find_hidden_triples;
pub use hidden_unique_rectangles::find_hidden_unique_rectangles;
pub use intersection_removals::find_intersection_removals;
pub use naked_singles::find_naked_singles;
pub use naked_tuples::find_naked_pairs;
pub use naked_tuples::find_naked_quads;
pub use naked_tuples::find_naked_triples;
pub use peers::find_peers;
pub use rectangle_eliminations::find_rectangle_eliminations;
pub use singles_chains::find_singles_chains;
pub use skyscrapers::find_skyscrapers;
pub use two_string_kites::find_two_string_kites;
pub use unique_rectangles::{find_almost_unique_rectangles, find_unique_rectangles};
pub use w_wings::find_w_wings;
pub use wxyz_wings::find_wxyz_wings;
pub use xy_chains::find_xy_chains;
pub use xyz_wings::find_xyz_wings;
pub use y_wings::find_y_wings;

use crate::layout::*;
use crate::puzzle::*;

mod avoidable_rectangles;
mod brute_force;
mod bugs;
mod extended_unique_rectangles;
mod fireworks;
mod fish;
mod hidden_singles;
mod hidden_tuples;
mod hidden_unique_rectangles;
mod intersection_removals;
mod naked_singles;
mod naked_tuples;
mod peers;
mod rectangle_eliminations;
mod singles_chains;
mod skyscrapers;
mod two_string_kites;
mod unique_rectangles;
mod w_wings;
mod wxyz_wings;
mod xy_chains;
mod xyz_wings;
mod y_wings;
