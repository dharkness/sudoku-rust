//! Provides the [`Parse`] and [`Format`] traits with several implementations.

pub mod format;
pub mod parse;
pub mod print;

pub use format::{
    format_for_console, format_for_fancy_console, format_for_url, format_for_wiki, format_packed,
    Format,
};
pub use parse::{Parse, ParsePacked};
pub use print::{print_candidate, print_candidates, print_values};
