//! Provides the [`Board`] for tracking the state of a puzzle,
//! the [`Action`]s that can be taken to solve it,
//! and any [`Error`]s that arise due to those actions.

pub mod action;
pub mod board;
pub mod effects;
pub mod error;
pub mod generate;
pub mod parse;
pub mod strategy;

pub use action::Action;
pub use board::{Board, CellState};
pub use effects::Effects;
pub use error::Error;
pub use generate::Generator;
pub use parse::Parser;
pub use strategy::Strategy;
