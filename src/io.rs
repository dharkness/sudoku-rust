//! Provides methods to [`Parse`] and [`Format`] puzzles and other I/O helpers.
//!
//! Use [`Parse`] to build a [`Board`][`crate::puzzle::Board`] from a string
//! and [`Format`] to produce a string from a board. This crate supports several
//! string formats for sharing puzzles.
//!
//! **Packed**
//!
//! The packed string format is an 81-character string with a digit for each
//! known cell and a period for each unknown cell. It cannot distinguish between
//! given and solved cells, but it is the most compact format. Parsing one will
//! ignore all other characters, and formatting one has an option to place
//! spaces between each row of 9 cells.
//!
//! ```
//! ...3.5.7. .48.....1 ...71.... .5...6... .......9. ....531.4 .9..8.... ..2....47 .8.....2.
//! ```
//!
//! **Wiki**
//!
//! This format was created Andrew Stuart of [SudokuWiki](https://www.sudokuwiki.org/)
//! to provide the full puzzle state: given clues, solved cells, and cell candidates.
//!
//! ```
//! 811003080g44g02044090g21g002441080444104g0108020030g080g201004098040g10202g04021100g05088104800840g0030h1120g0090402211080400h10400h8005082003g12102800h41g0090410
//! ```
//!
//! **Grid**
//!
//! This format is convenient for sharing puzzles in email or on forums.
//! Each lone digit is either a given or solved cell while each group of digits
//! represent a cell's remaining candidates.
//!
//! ```
//! +--------------------+-----------------------+-------------------+
//! | 1     257    457   | 9     6        458    | 478    3     2478 |
//! | 34679 3679   4679  | 2     13478    148    | 146789 4678  5    |
//! | 8     235679 45679 | 13457 13457    145    | 14679  2467  2479 |
//! +--------------------+-----------------------+-------------------+
//! | 4679  6789   1     | 3457  345789   4589   | 2      45678 4789 |
//! | 2479  2789   479   | 6     12345789 124589 | 345789 4578  4789 |
//! | 5     26789  3     | 47    24789    2489   | 46789  4678  1    |
//! +--------------------+-----------------------+-------------------+
//! | 3679  4      5679  | 8     259      2569   | 57     1     27   |
//! | 7     57     8     | 145   1245     3      | 457    9     6    |
//! | 69    1      2     | 45    459      7      | 458    458   3    |
//! +--------------------+-----------------------+-------------------+
//! ```
//!
//! [`Cancelable`] is used to detect when the user presses `Ctrl-C`
//! so a long-running process can be stopped without terminating the program.
//!
//! Finally, use [`show_progress`] to display a progress bar while building
//! or solving a puzzle and [`format_runtime`] and [`format_number`] for logging.

pub use cancelable::{create_signal, Cancelable};
pub use format::{format_for_fancy_console, format_for_wiki, format_grid, format_packed, Format};
pub use numbers::{format_number, format_runtime};
pub use parse::{Parse, ParsePacked, Parser};
pub use print::{print_candidate, print_candidates, print_givens, print_known_values};
pub use progress::show_progress;

mod cancelable;
mod format;
mod numbers;
mod parse;
mod print;
mod progress;

pub const SUDOKUWIKI_URL: &str = "https://www.sudokuwiki.org/sudoku.htm?bd=";
