//! Strategy-facing puzzle state and deduction plumbing.
//!
//! [`Board`] is the read-optimized view of a puzzle: solved cells, candidates,
//! and cached set views that make strategy scans cheap. Strategies should treat
//! it as immutable input and emit deductions as [`Action`]s and [`Effects`], not
//! mutate it directly.
//!
//! [`Action`] describes one logical step produced by a [`Strategy`], including
//! cell sets, candidate eliminations, and optional clue annotations for UI or
//! explanations. [`Effects`] aggregates actions and any [`Error`]s discovered
//! while applying them.
//!
//! [`Changer`] applies actions to a board according to [`Options`], including
//! optional automatic follow-on deductions such as singles or intersection
//! removals.
//!
//! [`PseudoCell`] models a composite cell for strategies that treat multiple
//! cells as one unit (for example, avoidable rectangles).
//!
//! See [`crate::layout`] for the geometry and bitset primitives used by
//! strategies and actions.

pub use action::Action;
pub use board::{Board, Change};
pub use changer::{ChangeResult, Changer};
pub use clues::{Clues, Verdict};
pub use effects::Effects;
pub use error::Error;
pub use options::Options;
pub use pseudo_cell::PseudoCell;
pub use strategy::{Difficulty, Strategy};

mod action;
mod board;
mod changer;
mod clues;
mod effects;
mod error;
mod options;
mod pseudo_cell;
mod strategy;
