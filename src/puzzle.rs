//! Defines the [`Board`] for tracking the current state of Sudoku puzzle
//! along with supporting elements for modifying it and reporting the effects that causes.
//!
//! Modifying a board by setting givens (starting clues), removing candidates, or solving cells
//! may result in follow-on [`Action`]s such as setting other cells or removing other candidates
//! and [`Error`]s such as cells or houses with no remaining candidates or deadly rectangles.
//! These are collected in an [`Effects`] object.
//!
//! A [`Changer`] may be used to apply actions to a board as well as any follow-on actions
//! configured to be applied automatically via its [`Options`]. At this time, the board
//! automatically removes candidates from all neighbor cells when a cell becomes known.
//! While this could be made optional, I doubt most players find erasing obvious pencil marks
//! to be the highlight of their playing time.
//!
//! [`Strategy`] enumerates all of the ways the board can be modified as well as the types
//! of deductions made by the various solving [`algorithms`][`crate::solve::algorithms`].
//!
//! Finally, a [`PseudoCell`] is used when two or more cells can be treated as a single cell
//! by a solving algorithm. Currently, only the Avoidable Rectangle strategy makes use of it,
//! but I suspect there are other strategies that could employ it to find more deductions.
//!
//! See the [`layout`][`crate::layout`] module for the individual pieces that make up the board.

//! Provides the [`Board`] for tracking the state of a puzzle,
//! the actions that can be taken to solve it,
//! and any errors that arise due to those actions.

pub use action::Action;
pub use board::Board;
pub use changer::{Change, Changer};
pub use effects::Effects;
pub use error::Error;
pub use options::Options;
pub use pseudo_cell::PseudoCell;
pub use strategy::Strategy;

mod action;
mod board;
mod changer;
mod effects;
mod error;
mod options;
mod pseudo_cell;
mod strategy;
