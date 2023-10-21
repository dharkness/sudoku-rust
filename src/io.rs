//! Provides the [`Parse`] and [`Format`] traits with several implementations.

mod cancelable;
mod format;
mod numbers;
mod parse;
mod print;
mod progress;

pub const SUDOKUWIKI_URL: &str = "https://www.sudokuwiki.org/sudoku.htm?bd=";

pub use cancelable::{create_signal, Cancelable};
pub use format::{format_for_fancy_console, format_for_wiki, format_grid, format_packed, Format};
pub use numbers::{format_number, format_runtime};
pub use parse::{Parse, ParseGrid, ParsePacked, ParseWiki, Parser};
pub use print::{print_candidate, print_candidates, print_givens, print_known_values};
pub use progress::show_progress;
