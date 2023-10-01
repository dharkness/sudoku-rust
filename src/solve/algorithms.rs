use itertools::Itertools;

use crate::layout::*;
use crate::puzzle::*;

mod avoidable_rectangles;
mod bugs;
mod empty_rectangles;
mod fish;
mod hidden_tuples;
mod intersection_removals;
mod naked_tuples;
mod singles_chains;
mod skyscrapers;
mod unique_rectangles;
mod xy_chains;
mod xyz_wings;
mod y_wings;

pub use avoidable_rectangles::find_avoidable_rectangles;
pub use bugs::find_bugs;
pub use empty_rectangles::find_empty_rectangles;
pub use fish::find_jellyfish;
pub use fish::find_swordfish;
pub use fish::find_x_wings;
pub use hidden_tuples::find_hidden_pairs;
pub use hidden_tuples::find_hidden_quads;
pub use hidden_tuples::find_hidden_triples;
pub use intersection_removals::find_intersection_removals;
pub use naked_tuples::find_naked_pairs;
pub use naked_tuples::find_naked_quads;
pub use naked_tuples::find_naked_triples;
pub use singles_chains::find_singles_chains;
pub use skyscrapers::find_skyscrapers;
pub use unique_rectangles::find_unique_rectangles;
pub use xy_chains::find_xy_chains;
pub use xyz_wings::find_xyz_wings;
pub use y_wings::find_y_wings;
