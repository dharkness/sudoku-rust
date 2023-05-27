use crate::layout::Board;

pub mod deadly_rectangles;
pub mod intersection_removals;

pub type Solver = fn(&Board) -> Option<Board>;