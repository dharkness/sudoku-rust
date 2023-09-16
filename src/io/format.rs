use crate::layout::House;
use crate::puzzle::Board;
use crate::symbols::MISSING;

/// Formats a [`Board`] into a packed string with spacing and periods for unsolved cells.
pub fn format_for_console(board: &Board) -> String {
    PackedFormat::console().format(board)
}

/// Formats a [`Board`] into a packed string with spacing and Unicode dots for unsolved cells.
pub fn format_for_fancy_console(board: &Board) -> String {
    PackedFormat::fancy().format(board)
}

/// Formats a [`Board`] into a packed string with zeros for unsolved cells.
pub fn format_for_url(board: &Board) -> String {
    PackedFormat::url().format(board)
}

/// Formats a [`Board`] into a packed string for the SudokuWiki site.
pub fn format_for_wiki(board: &Board) -> String {
    WikiFormat::new().format(board)
}

/// Formats a [`Board`] into a packed string.
pub fn format_packed(board: &Board, unknown: char, spaces: bool) -> String {
    PackedFormat::new(unknown, spaces).format(board)
}

/// Produces a single-line packed string of the [`Board`]'s cells
/// with a configured character for all unsolved cells
/// and optional space separating rows.
pub struct PackedFormat {
    pub unknown: char,
    pub spaces: bool,
}

impl PackedFormat {
    pub const fn new(unknown: char, spaces: bool) -> Self {
        PackedFormat { unknown, spaces }
    }

    pub const fn console() -> Self {
        PackedFormat::new('.', true)
    }

    pub const fn url() -> Self {
        PackedFormat::new('0', false)
    }

    pub const fn fancy() -> Self {
        PackedFormat::new(MISSING, true)
    }

    fn format(&self, board: &Board) -> String {
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

/// Produces a single-line packed string of the [`Board`]'s cells for SudokuWiki
/// detailing givens, solved cells, and unsolved candidates.
pub struct WikiFormat {
    pub spaces: bool,
}

impl WikiFormat {
    pub const fn new() -> Self {
        WikiFormat { spaces: false }
    }

    pub const fn new_with_spaces() -> Self {
        WikiFormat { spaces: true }
    }

    fn format(&self, board: &Board) -> String {
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
    use crate::io::parse::parse;
    use crate::io::Parser;

    #[test]
    fn test_format_for_console() {
        let board = parse(
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
        let board = parse(
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
            format_packed(&board, '-', false)
        );
    }

    #[test]
    fn test_format_for_wiki() {
        let parser = Parser::new(true, true);
        let (mut board, _, _) = parser.parse(
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
