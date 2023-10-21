use itertools::Itertools;

use crate::layout::{Cell, Known, KnownSet};
use crate::puzzle::{Board, Change, Changer, Effects, Options, Strategy};

/// Provides helper methods for parsing puzzle strings into boards.
pub struct Parse {}

impl Parse {
    /// Returns a new [`ParsePacked`] that ignores errors
    /// and won't solve hidden/naked single automatically.
    pub fn packed() -> ParsePacked {
        ParsePacked::new()
    }

    /// Returns a new [`ParsePacked`] with the given options.
    pub fn packed_with_options(options: Options) -> ParsePacked {
        ParsePacked::new_with_options(options)
    }

    /// Returns a new [`ParsePacked`] with the given changer.
    pub fn packed_with_player(changer: Changer) -> ParsePacked {
        ParsePacked::new_with_player(changer)
    }

    /// Returns a new [`ParseGrid`] that ignores errors.
    pub fn grid() -> ParseGrid {
        ParseGrid::new()
    }

    /// Returns a new [`ParseWiki`] that ignores errors.
    pub fn wiki() -> ParseWiki {
        ParseWiki::new()
    }
}

/// Parses puzzle strings into boards, optionally stopping on errors
/// and/or automatically solving naked and hidden singles.
#[derive(Default)]
pub struct ParsePacked {
    pub changer: Changer,
}

impl ParsePacked {
    pub fn new() -> Self {
        ParsePacked::default()
    }

    pub fn new_with_options(options: Options) -> Self {
        ParsePacked::new_with_player(Changer::new(options))
    }

    pub fn new_with_player(changer: Changer) -> ParsePacked {
        ParsePacked { changer }
    }

    /// Builds a new board using an input string to set some cells,
    /// and returns it without any actions or errors that arise.
    pub fn parse_simple(&self, input: &str) -> Board {
        self.parse(input).0
    }

    /// Builds a new board using an input string to set some cells,
    /// and returns it along with any actions and errors that arise.
    ///
    /// - Use a digit (1 to 9) to set a cell's value.
    /// - Use whitespace, pipes, and underscores for readability.
    /// - Use any other character to leave a cell unsolved.
    pub fn parse(&self, input: &str) -> (Board, Effects, Option<(Cell, Known)>) {
        let mut board = Board::new();
        let mut unapplied = Effects::new();
        let mut c = 0;

        for char in input.chars() {
            match char {
                ' ' | '\r' | '\n' | '|' | '_' => continue,
                '1'..='9' => {
                    let cell = Cell::new(c);
                    let known = Known::from(char);
                    let current = board.value(cell);
                    if current != known.value() {
                        match self.changer.set_given(&board, Strategy::Given, cell, known) {
                            Change::None => (),
                            Change::Valid(after, actions) => {
                                board = *after;
                                unapplied.take_actions(actions);
                            }
                            Change::Invalid(before, _, _, mut errors) => {
                                if self.changer.options.stop_on_error {
                                    errors.take_actions(unapplied);
                                    return (*before, errors, Some((cell, known)));
                                }
                            }
                        }
                    }
                }
                _ => (),
            }

            c += 1;
        }

        (board, unapplied, None)
    }
}

/// Parses puzzle strings into boards with the exact solved cells and candidates
/// from the grid format.
#[derive(Default)]
pub struct ParseGrid {
    stop_on_error: bool,
}

impl ParseGrid {
    pub fn new() -> Self {
        ParseGrid::default()
    }

    /// Sets the parser to stop on the first error.
    pub fn stop_on_error(mut self) -> Self {
        self.stop_on_error = true;
        self
    }

    /// Builds a new board using an input string to set some cells,
    /// and returns it without any actions or errors that arise.
    pub fn parse_simple(&self, input: &str) -> Board {
        self.parse(input).0
    }

    /// Builds a new board using an input string to set some cells,
    /// and returns it along with any actions and errors that arise.
    pub fn parse(&self, input: &str) -> (Board, Effects, Option<(Cell, Known)>) {
        let mut board = Board::new();
        let mut effects = Effects::new();

        let mut candidates = [KnownSet::empty(); 81];
        let mut c: usize = 0;
        let mut collecting = false;
        for char in input.chars() {
            if ('1'..='9').contains(&char) {
                collecting = true;
                candidates[c] += Known::from(char);
            } else if collecting {
                collecting = false;
                c += 1;
                if c >= 81 {
                    break;
                }
            }
        }

        for (c, knowns) in candidates.iter().enumerate() {
            let cell = Cell::new(c as u8);

            if let Some(solved) = knowns.as_single() {
                board.set_known(cell, solved, &mut effects);
                if effects.has_errors() && self.stop_on_error {
                    return (board, effects, Some((cell, solved)));
                }
                effects.clear_actions();
            } else {
                for known in knowns.inverted() {
                    if board.remove_candidate(cell, known, &mut effects) {
                        if effects.has_errors() && self.stop_on_error {
                            return (board, effects, Some((cell, known)));
                        }
                        effects.clear_actions();
                    }
                }
            }
        }

        (board, effects, None)
    }
}

/// Parses puzzle strings into boards with the exact given/solved cells and candidates.
///
/// See https://www.sudokuwiki.org/Sudoku_String_Definitions for more information.
#[derive(Default)]
pub struct ParseWiki {
    stop_on_error: bool,
}

impl ParseWiki {
    pub fn new() -> Self {
        ParseWiki::default()
    }

    /// Sets the parser to stop on the first error.
    pub fn stop_on_error(mut self) -> Self {
        self.stop_on_error = true;
        self
    }

    /// Builds a new board using an input string to set some cells,
    /// and returns it without any actions or errors that arise.
    pub fn parse_simple(&self, input: &str) -> Board {
        self.parse(input).0
    }

    /// Builds a new board using an input string to set some cells,
    /// and returns it along with any actions and errors that arise.
    pub fn parse(&self, input: &str) -> (Board, Effects, Option<(Cell, Known)>) {
        let mut board = Board::new();
        let mut effects = Effects::new();

        for (c, chars) in input.chars().collect::<Vec<char>>().chunks(2).enumerate() {
            if chars.len() != 2 {
                break;
            }
            let value = 32 * to_decimal(chars[0]) + to_decimal(chars[1]);
            if value > 1022 {
                break;
            }

            let cell = Cell::new(c as u8);
            let given = value % 2 == 1;
            let knowns = KnownSet::new(value >> 1);

            if let Some(solved) = knowns.as_single() {
                if given {
                    board.set_given(cell, solved, &mut effects)
                } else {
                    board.set_known(cell, solved, &mut effects)
                };
                if effects.has_errors() && self.stop_on_error {
                    return (board, effects, Some((cell, solved)));
                }
                effects.clear_actions();
            } else {
                if given {
                    break;
                }
                for known in knowns.inverted() {
                    board.remove_candidate(cell, known, &mut effects);
                    if effects.has_errors() && self.stop_on_error {
                        return (board, effects, Some((cell, known)));
                    }
                    effects.clear_actions();
                }
            }
        }

        (board, effects, None)
    }
}

fn to_decimal(c: char) -> u16 {
    match c {
        '0'..='9' => c as u16 - '0' as u16,
        'A'..='Z' => c as u16 - 'A' as u16 + 10,
        'a'..='z' => c as u16 - 'a' as u16 + 10,
        _ => 0,
    }
}

fn trim_grid_whitespace(input: &str) -> String {
    input
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::format::{format_for_console, format_grid};
    use crate::io::format_for_wiki;

    #[test]
    fn test_parse_packed() {
        let parser = Parse::packed_with_options(Options::all());
        let (board, effects, failed) = parser.parse(
            "
            .1..7....
            2...4....
            .7.3.59..
            .29...4.5
            1..4.....
            ...9....2
            6..8.....
            952....1.
            ....6..7.
        ",
        );
        assert!(failed.is_none());
        assert!(!effects.has_errors());

        let (want, effects, failed) = parser.parse(
            "
            51.279.4.
            29.1465.7
            476385921
            .2961.4.5
            16542.79.
            .8495.162
            637891254
            952734.1.
            841562379
        ",
        );
        assert!(failed.is_none());
        assert!(!effects.has_errors());

        assert_eq!(format_for_console(&want), format_for_console(&board))
    }

    #[test]
    fn test_parse_grid() {
        let parser = Parse::grid().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "
                +---------------+-----------------+--------------+
                | 48  9   2     | 145   18   158  | 3   7   6    |
                | 478 1   468   | 24679 3    2689 | 5   248 248  |
                | 3   567 4568  | 24567 2678 2568 | 1   9   248  |
                +---------------+-----------------+--------------+
                | 9   3   46    | 8     5    26   | 7   24  1    |
                | 78  567 1568  | 3     126  4    | 689 258 2589 |
                | 2   56  14568 | 16    9    7    | 68  458 3    |
                +---------------+-----------------+--------------+
                | 6   8   9     | 257   27   3    | 4   1   57   |
                | 5   2   3     | 179   4    189  | 89  6   789  |
                | 1   4   7     | 569   68   5689 | 2   3   589  |
                +---------------+-----------------+--------------+
            ",
        );
        assert!(failed.is_none());
        assert!(!effects.has_errors());

        assert_eq!(
            "8gg0041i8292084020cg02agmk08q4108k8k0870bg7ke4b402g08kg0082g801024400k02c070b208260gq094p40430bi22g040a09g082080g05444080g0250100408k20go2o020s0020g40j0a0r00408p0",
            format_for_wiki(&board)
        );
    }

    #[test]
    fn test_parse_wiki() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "8gg0051i8292094121cg03agmk09q4118k8k0870bg7ke4b402g18kg1082g811124400k03c070b209260hq094p40530bi22g141a09g092081g05444080g0250100409k20ho2o021s0030h41j0a0r00508p0",
        );
        assert!(failed.is_none());
        assert!(!effects.has_errors());

        let want = trim_grid_whitespace(
            "
            +---------------+-----------------+--------------+
            | 48  9   2     | 145   18   158  | 3   7   6    |
            | 478 1   468   | 24679 3    2689 | 5   248 248  |
            | 3   567 4568  | 24567 2678 2568 | 1   9   248  |
            +---------------+-----------------+--------------+
            | 9   3   46    | 8     5    26   | 7   24  1    |
            | 78  567 1568  | 3     126  4    | 689 258 2589 |
            | 2   56  14568 | 16    9    7    | 68  458 3    |
            +---------------+-----------------+--------------+
            | 6   8   9     | 257   27   3    | 4   1   57   |
            | 5   2   3     | 179   4    189  | 89  6   789  |
            | 1   4   7     | 569   68   5689 | 2   3   589  |
            +---------------+-----------------+--------------+
        ",
        );

        assert_eq!(want, format_grid(&board));
    }
}
