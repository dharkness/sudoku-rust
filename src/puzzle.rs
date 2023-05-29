//! Provides the [`Board`] for tracking the state of a puzzle,
//! the [`Action`]s that can be taken to solve it,
//! and any [`Error`]s that arise due to those actions.

mod action;
mod board;
mod effects;
mod error;
mod generate;
mod strategy;

pub use action::Action;
pub use board::Board;
pub use effects::Effects;
pub use error::Error;
pub use generate::Generator;
pub use strategy::Strategy;
