use itertools::Itertools;

use crate::layout::{House, KnownSet};
use crate::puzzle::Board;
use crate::symbols::MISSING;

/// Formats a [`Board`] into a packed string with spacing and periods for unsolved cells.
pub fn format_for_console(board: &Board) -> String {
    Format::console().format(board)
}

/// Formats a [`Board`] into a packed string with spacing and Unicode dots for unsolved cells.
pub fn format_for_fancy_console(board: &Board) -> String {
    Format::fancy().format(board)
}

/// Formats a [`Board`] into a packed string with zeros for unsolved cells.
pub fn format_for_url(board: &Board) -> String {
    Format::url().format(board)
}

/// Formats a [`Board`] into a packed string for the SudokuWiki site.
pub fn format_for_wiki(board: &Board) -> String {
    Format::wiki().format(board)
}

/// Formats a [`Board`] into a packed string.
pub fn format_packed(board: &Board, unknown: char, spaces: bool) -> String {
    let mut formatter = FormatPacked::new(unknown);
    if spaces {
        formatter = formatter.spaces();
    }
    formatter.format(board)
}

/// Formats a [`Board`] into an ASCII grid showing all knowns and candidates.
pub fn format_grid(board: &Board) -> String {
    Format::grid().format(board)
}

/// Provides helper methods for parsing puzzle strings into [`Board`]s.
pub struct Format {}

impl Format {
    /// Formats a [`Board`] into a packed string with spacing and periods for unsolved cells.
    pub const fn console() -> FormatPacked {
        FormatPacked::new('.').spaces()
    }

    pub const fn fancy() -> FormatPacked {
        FormatPacked::new(MISSING).spaces()
    }

    pub const fn url() -> FormatPacked {
        FormatPacked::new('0')
    }

    pub const fn wiki() -> FormatWiki {
        FormatWiki::new()
    }

    pub const fn grid() -> FormatGrid {
        FormatGrid::new()
    }
}

/// Produces a single-line packed string of the [`Board`]'s cells
/// with a configured character for all unsolved cells
/// and optional space separating rows.
pub struct FormatPacked {
    pub unknown: char,
    pub spaces: bool,
}

impl FormatPacked {
    pub const fn new(unknown: char) -> Self {
        FormatPacked {
            unknown,
            spaces: false,
        }
    }

    /// Changes the character used for unsolved cells.
    pub const fn unknown(mut self, unknown: char) -> Self {
        self.unknown = unknown;
        self
    }

    /// Adds a space between rows.
    pub const fn spaces(mut self) -> Self {
        self.spaces = true;
        self
    }

    pub fn format(&self, board: &Board) -> String {
        let mut result = String::new();

        House::rows_iter().for_each(|row| {
            if self.spaces {
                result += " ";
            }
            row.cells().iter().for_each(|cell| {
                let value = board.value(cell);
                if value.is_known() {
                    result.push(value.label());
                } else {
                    result.push(self.unknown);
                }
            })
        });

        if self.spaces {
            result[1..].to_string()
        } else {
            result
        }
    }
}

/// Produces a ASCII grid of the [`Board`]'s cells for emailing
/// showing the solution or candidates for each cell.
#[derive(Default)]
pub struct FormatGrid {}

impl FormatGrid {
    pub const fn new() -> Self {
        FormatGrid {}
    }

    pub fn format(&self, board: &Board) -> String {
        let mut border = String::new();
        let mut rows: [String; 9] = Default::default();
        let widths = House::columns_iter()
            .map(|column| {
                column
                    .cells()
                    .iter()
                    .map(|cell| {
                        if board.is_known(cell) {
                            1
                        } else {
                            board.candidates(cell).len()
                        }
                    })
                    .max()
                    .unwrap()
            })
            .collect_vec();

        House::columns_iter().for_each(|column| {
            if column.is_block_left() {
                border += "+-";
            }
            border += "-----------"[..widths[column.usize()] + 1].into();
            if column.is_right() {
                border += "+";
            }

            column.cells().iter().enumerate().for_each(|(r, cell)| {
                let candidates = if board.is_known(cell) {
                    KnownSet::of(board.value(cell).known().unwrap())
                } else {
                    board.candidates(cell)
                };

                let row = &mut rows[r];
                if column.is_block_left() {
                    *row += "| ";
                }
                *row += &format!(
                    "{:width$} ",
                    candidates
                        .iter()
                        .map(|known| known.label())
                        .collect::<String>(),
                    width = widths[column.usize()]
                );
                if column.is_right() {
                    *row += "|";
                }
            })
        });

        vec![
            border.as_str(),
            rows[0].as_str(),
            rows[1].as_str(),
            rows[2].as_str(),
            border.as_str(),
            rows[3].as_str(),
            rows[4].as_str(),
            rows[5].as_str(),
            border.as_str(),
            rows[6].as_str(),
            rows[7].as_str(),
            rows[8].as_str(),
            border.as_str(),
        ]
        .join("\n")
    }
}

/// Produces a single-line packed string of the [`Board`]'s cells for SudokuWiki
/// detailing givens, solved cells, and unsolved candidates.
///
/// See https://www.sudokuwiki.org/Sudoku_String_Definitions for more information.
#[derive(Default)]
pub struct FormatWiki {
    pub spaces: bool,
}

impl FormatWiki {
    pub const fn new() -> Self {
        FormatWiki { spaces: false }
    }

    /// Adds a space between rows.
    pub const fn spaces(mut self) -> Self {
        self.spaces = true;
        self
    }

    pub fn format(&self, board: &Board) -> String {
        let mut result = String::new();

        House::rows_iter().for_each(|row| {
            if self.spaces {
                result += " ";
            }
            row.cells().iter().for_each(|cell| {
                let mut value: u32;
                if board.is_known(cell) {
                    value = 1 << board.value(cell).value();
                    if board.is_given(cell) {
                        value += 1;
                    }
                } else {
                    value = (board.candidates(cell).bits() << 1) as u32;
                }

                if value < 32 {
                    result.push('0');
                    result.push(std::char::from_digit(value, 32).unwrap());
                } else {
                    result.push(std::char::from_digit(value / 32, 32).unwrap());
                    result.push(std::char::from_digit(value % 32, 32).unwrap());
                }
            })
        });

        if self.spaces {
            result[1..].to_string()
        } else {
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::{Parse, Parser};
    use crate::puzzle::Options;
    use crate::testing::strip_leading_whitespace;

    #[test]
    fn test_format_for_console() {
        let board = Parse::packed_with_options(Options::errors_and_peers()).parse_simple(
            "
                .8.1.3.7.
                .9.5.6...
                ..14.8.2.
                578241639
                143659782
                926837451
                .379.52..
                ...3.4.97
                419782.6.
            ",
        );

        assert_eq!(
            ".8.1.3.7. .9.5.6... ..14.8.2. 578241639 143659782 926837451 .379.52.. ...3.4.97 419782.6.",
            format_for_console(&board)
        );
    }

    #[test]
    fn test_format_packed() {
        let board = Parse::packed_with_options(Options::errors_and_peers()).parse_simple(
            "
                .8.1.3.7.
                .9.5.6...
                ..14.8.2.
                578241639
                143659782
                926837451
                .379.52..
                ...3.4.97
                419782.6.
            ",
        );

        assert_eq!(
            "-8-1-3-7--9-5-6-----14-8-2-578241639143659782926837451-379-52-----3-4-97419782-6-",
            FormatPacked::new('-').format(&board)
        );
    }

    #[test]
    fn test_format_grid() {
        let board = Parse::packed_with_options(Options::errors_and_peers()).parse_simple(
            "
                ..2...376
                .1..3.5..
                .......9.
                9..85...1
                ...3.4...
                2...97..3
                .8.......
                ..3.4..6.
                147...2..
            ",
        );

        let want = strip_leading_whitespace(
            "
            +-------------------+---------------------+-----------------+
            | 458    59   2     | 1459   18    1589   | 3    7    6     |
            | 4678   1    4689  | 24679  3     2689   | 5    248  248   |
            | 345678 3567 4568  | 124567 12678 12568  | 148  9    248   |
            +-------------------+---------------------+-----------------+
            | 9      367  46    | 8      5     26     | 467  24   1     |
            | 5678   567  1568  | 3      126   4      | 6789 258  25789 |
            | 2      56   14568 | 16     9     7      | 468  458  3     |
            +-------------------+---------------------+-----------------+
            | 56     8    569   | 125679 1267  123569 | 1479 1345 4579  |
            | 5      259  3     | 12579  4     12589  | 1789 6    5789  |
            | 1      4    7     | 569    68    35689  | 2    358  589   |
            +-------------------+---------------------+-----------------+
        ",
        );

        assert_eq!(want, format_grid(&board));
    }

    #[test]
    fn test_format_for_wiki() {
        let board = Parse::packed_with_options(Options::all().return_intersection_removals())
            .parse_simple(
                "
                ..2...376
                .1..3.5..
                .......9.
                9..85...1
                ...3.4...
                2...97..3
                .8.......
                ..3.4..6.
                147...2..
            ",
            );

        assert_eq!(
            "8gg0051i8292094121cg03agmk09q4118k8k0870bg7ke4b402g18kg1082g811124400k03c070b209260hq094p40530bi22g141a09g092081g05444080g0250100409k20ho2o021s0030h41j0a0r00508p0",
            format_for_wiki(&board)
        );
    }
}
