use crate::layout::{Cell, Known};
use crate::puzzle::{Board, Effects, Error, Strategy};

/// Parses puzzle strings into [`Board`]s, optionally stopping on errors
/// and/or automatically solving naked and hidden singles.
pub struct Parser {
    stop_on_error: bool,
    solve_singles: bool,
}

impl Parser {
    pub fn new(stop_on_error: bool, solve_singles: bool) -> Self {
        Parser {
            stop_on_error,
            solve_singles,
        }
    }

    /// Builds a new [`Board`] using an input string to set some cells,
    /// and returns it along with any [`Action`]s and [`Error`]s that arise.
    ///
    /// - Use a digit (1 to 9) to set a cell's value.
    /// - Use whitespace, pipes, and underscores for readability.
    /// - Use any other character to leave a cell unsolved.
    pub fn parse(&self, input: &str) -> (Board, Effects, Option<(Cell, Known)>) {
        let mut board = Board::new();
        let mut effects = Effects::new();
        let mut c = 0;

        for char in input.chars() {
            match char {
                ' ' | '\r' | '\n' | '|' | '_' => continue,
                '1'..='9' => {
                    let cell = Cell::new(c);
                    let known = Known::from(char);
                    let current = board.value(cell);
                    if current != known.value() {
                        if board.is_candidate(cell, known) {
                            board.set_known(cell, known, &mut effects);
                            if effects.has_errors() && self.stop_on_error {
                                return (board, effects, Some((cell, known)));
                            }
                            if self.solve_singles {
                                effects.apply_all(&mut board);
                            } else {
                                effects.apply_all_strategy(&mut board, Strategy::Neighbor);
                            }
                            effects.clear_actions();
                        } else if self.stop_on_error {
                            if current.is_known() {
                                effects.add_error(Error::AlreadySolved(
                                    cell,
                                    known,
                                    current.known(),
                                ));
                            } else {
                                effects.add_error(Error::NotCandidate(cell, known));
                            }
                            return (board, effects, Some((cell, known)));
                        }
                    }
                }
                _ => (),
            }

            c += 1;
        }

        (board, effects, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let parser = Parser::new(true, true);
        let (board, errors, failed) = parser.parse(
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
        assert!(!errors.has_errors());

        let (want, errors, failed) = parser.parse(
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
        assert!(!errors.has_errors());

        assert_eq!(want, board)
    }
}
