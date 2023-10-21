//! Provides the [`Board`] for tracking the state of a puzzle,
//! the actions that can be taken to solve it,
//! and any errors that arise due to those actions.

mod action;
mod board;
mod changer;
mod effects;
mod error;
mod options;
mod pseudo_cell;
mod strategy;

pub use action::Action;
pub use board::Board;
pub use changer::{Change, Changer};
pub use effects::Effects;
pub use error::Error;
pub use options::Options;
pub use pseudo_cell::PseudoCell;
pub use strategy::Strategy;
