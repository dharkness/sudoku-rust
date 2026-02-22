use std::collections::VecDeque;

use super::*;

const DIGIT_COUNT: usize = Digit::COUNT as usize;
const CELL_COUNT: usize = Cell::COUNT as usize;
const NODE_COUNT: usize = DIGIT_COUNT * CELL_COUNT;

pub fn find_digit_forcing_chains(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();
    let links = Links::new(board);

    for cell in Cell::iter() {
        let digits = board.candidates(cell);
        if digits.len() < 2 {
            continue;
        }
        for digit in digits {
            let start = node_index(cell, digit);
            let on = propagate(&links, start, true);
            let off = propagate(&links, start, false);
            if on.contradiction || off.contradiction {
                continue;
            }

            let mut action = Action::new(Strategy::DigitForcingChain);
            action.clue_cell_for_digit(Verdict::Primary, cell, digit);

            add_candidate_conclusions(board, &links, &on, &off, &mut action);
            add_cell_conclusions(board, &on, &off, &mut action);
            add_unit_conclusions(board, &links, &on, &off, &mut action);

            if action.is_empty() {
                continue;
            }

            if effects.add_action(action) && single {
                return Some(effects);
            }
        }
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

fn add_candidate_conclusions(
    board: &Board,
    links: &Links,
    on: &Implications,
    off: &Implications,
    action: &mut Action,
) {
    for node in 0..NODE_COUNT {
        if !links.present[node] {
            continue;
        }
        let (cell, digit) = node_to_cell_digit(node);
        if on.on[node] && off.on[node] {
            if board.value(cell).digit() != Some(digit) {
                action.set(cell, digit);
            }
            action.clue_cell_for_digit(Verdict::Secondary, cell, digit);
        } else if on.off[node] && off.off[node] {
            if board.candidates(cell).has(digit) {
                action.erase(cell, digit);
            }
            action.clue_cell_for_digit(Verdict::Secondary, cell, digit);
        }
    }
}

fn add_cell_conclusions(board: &Board, on: &Implications, off: &Implications, action: &mut Action) {
    for cell in Cell::iter() {
        let candidates = board.candidates(cell);
        if candidates.len() < 2 {
            continue;
        }

        let mut on_digits = DigitSet::empty();
        let mut off_digits = DigitSet::empty();
        let mut on_from_on = DigitSet::empty();
        let mut on_from_off = DigitSet::empty();
        let mut off_from_on = DigitSet::empty();
        let mut off_from_off = DigitSet::empty();

        for digit in candidates {
            let node = node_index(cell, digit);
            if on.on[node] {
                on_digits += digit;
                on_from_on += digit;
            }
            if off.on[node] {
                on_digits += digit;
                on_from_off += digit;
            }
            if on.off[node] {
                off_digits += digit;
                off_from_on += digit;
            }
            if off.off[node] {
                off_digits += digit;
                off_from_off += digit;
            }
        }

        if !on_from_on.is_empty() && !on_from_off.is_empty() {
            let remove = candidates - on_digits;
            if !remove.is_empty() {
                action.erase_digits(cell, remove);
                action.clue_cell_for_digits(Verdict::Secondary, cell, on_digits);
            }
        }

        if candidates.len() == 3 && !off_from_on.is_empty() && !off_from_off.is_empty() {
            let remaining = candidates - off_digits;
            if remaining.len() == 1 {
                if let Some(digit) = remaining.as_single() {
                    action.set(cell, digit);
                    action.clue_cell_for_digits(Verdict::Secondary, cell, candidates);
                }
            }
        }
    }
}

fn add_unit_conclusions(
    board: &Board,
    links: &Links,
    on: &Implications,
    off: &Implications,
    action: &mut Action,
) {
    for digit in Digit::iter() {
        for house in House::iter() {
            let candidates = board.house_candidate_cells(house, digit);
            if candidates.len() < 2 {
                continue;
            }

            let mut on_cells = CellSet::empty();
            let mut on_from_on = CellSet::empty();
            let mut on_from_off = CellSet::empty();
            let mut off_cells = CellSet::empty();
            let mut off_from_on = CellSet::empty();
            let mut off_from_off = CellSet::empty();

            for cell in candidates {
                let node = node_index(cell, digit);
                if on.on[node] {
                    on_cells.add(cell);
                    on_from_on.add(cell);
                }
                if off.on[node] {
                    on_cells.add(cell);
                    on_from_off.add(cell);
                }
                if on.off[node] {
                    off_cells.add(cell);
                    off_from_on.add(cell);
                }
                if off.off[node] {
                    off_cells.add(cell);
                    off_from_off.add(cell);
                }
            }

            if !on_from_on.is_empty() && !on_from_off.is_empty() {
                let remove = candidates - on_cells;
                if !remove.is_empty() {
                    action.erase_cells(remove, digit);
                    action.clue_cells_for_digit(Verdict::Secondary, on_cells, digit);
                }
            }

            if candidates.len() == 3 && !off_from_on.is_empty() && !off_from_off.is_empty() {
                let remaining = candidates - off_cells;
                if remaining.len() == 1 {
                    if let Some(cell) = remaining.as_single() {
                        if links.present[node_index(cell, digit)] {
                            action.set(cell, digit);
                            action.clue_cells_for_digit(Verdict::Secondary, candidates, digit);
                        }
                    }
                }
            }
        }
    }
}

struct Links {
    strong: Vec<Vec<usize>>,
    weak: Vec<Vec<usize>>,
    present: Vec<bool>,
}

impl Links {
    fn new(board: &Board) -> Self {
        let mut strong = vec![Vec::new(); NODE_COUNT];
        let mut weak = vec![Vec::new(); NODE_COUNT];
        let mut present = vec![false; NODE_COUNT];

        for cell in Cell::iter() {
            let candidates = board.candidates(cell);
            for digit in candidates {
                present[node_index(cell, digit)] = true;
            }

            let digits: Vec<Digit> = candidates.iter().collect();
            for i in 0..digits.len() {
                for j in (i + 1)..digits.len() {
                    let a = node_index(cell, digits[i]);
                    let b = node_index(cell, digits[j]);
                    add_edge(&mut weak, a, b);
                }
            }

            if digits.len() == 2 {
                let a = node_index(cell, digits[0]);
                let b = node_index(cell, digits[1]);
                add_edge(&mut strong, a, b);
            }
        }

        for digit in Digit::iter() {
            for house in House::iter() {
                let cells = board.house_candidate_cells(house, digit);
                let cell_vec: Vec<Cell> = cells.iter().collect();
                for i in 0..cell_vec.len() {
                    for j in (i + 1)..cell_vec.len() {
                        let a = node_index(cell_vec[i], digit);
                        let b = node_index(cell_vec[j], digit);
                        add_edge(&mut weak, a, b);
                    }
                }

                if cell_vec.len() == 2 {
                    let a = node_index(cell_vec[0], digit);
                    let b = node_index(cell_vec[1], digit);
                    add_edge(&mut strong, a, b);
                }
            }
        }

        for index in 0..NODE_COUNT {
            strong[index].sort_unstable();
            strong[index].dedup();
            weak[index].sort_unstable();
            weak[index].dedup();
        }

        Links {
            strong,
            weak,
            present,
        }
    }
}

#[derive(Clone)]
struct Implications {
    on: Vec<bool>,
    off: Vec<bool>,
    contradiction: bool,
}

fn propagate(links: &Links, start: usize, start_on: bool) -> Implications {
    let mut on = vec![false; NODE_COUNT];
    let mut off = vec![false; NODE_COUNT];
    let mut contradiction = false;
    let mut queue: VecDeque<(usize, bool)> = VecDeque::new();

    if start_on {
        on[start] = true;
    } else {
        off[start] = true;
    }
    queue.push_back((start, start_on));

    while let Some((node, is_on)) = queue.pop_front() {
        if is_on {
            for &neighbor in &links.weak[node] {
                if off[neighbor] {
                    continue;
                }
                if on[neighbor] {
                    contradiction = true;
                    continue;
                }
                off[neighbor] = true;
                queue.push_back((neighbor, false));
            }
        } else {
            for &neighbor in &links.strong[node] {
                if on[neighbor] {
                    continue;
                }
                if off[neighbor] {
                    contradiction = true;
                    continue;
                }
                on[neighbor] = true;
                queue.push_back((neighbor, true));
            }
        }
    }

    Implications {
        on,
        off,
        contradiction,
    }
}

fn add_edge(edges: &mut [Vec<usize>], a: usize, b: usize) {
    edges[a].push(b);
    edges[b].push(a);
}

fn node_index(cell: Cell, digit: Digit) -> usize {
    cell.usize() * DIGIT_COUNT + digit.usize()
}

fn node_to_cell_digit(index: usize) -> (Cell, Digit) {
    let cell = Cell::new((index / DIGIT_COUNT) as u8);
    let digit = Digit::new((index % DIGIT_COUNT) as u32);
    (cell, digit)
}

#[cfg(test)]
mod tests {
    use crate::io::{Parse, Parser};
    use crate::puzzle::Options;
    use crate::*;

    use super::*;

    fn assert_any_erase(actions: &[Action], cell: Cell, digit: Digit) {
        assert!(
            actions.iter().any(|action| action.erases(cell, digit)),
            "expected erase of {} from {}",
            digit,
            cell
        );
    }

    fn assert_any_set(actions: &[Action], cell: Cell, digit: Digit) {
        assert!(
            actions.iter().any(|action| action.sets(cell, digit)),
            "expected set of {} in {}",
            digit,
            cell
        );
    }

    #[test]
    fn figure_1_example() {
        let parser = Parse::packed_with_options(Options::errors());
        let (board, effects, failed) = parser.parse(
            "000060000007009501904370800300007258000000000785000003000082607206700980070090000",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_digit_forcing_chains(&board, false).expect("not found");
        assert_any_erase(got.actions(), cell!(E2), digit!(6));
        assert_any_erase(got.actions(), cell!(E8), digit!(6));
    }

    #[test]
    fn figure_2_example() {
        let parser = Parse::packed_with_options(Options::errors());
        let (board, effects, failed) = parser.parse(
            "000060000007009501904370800300007258000000000785000003000082607206700980070090000",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_digit_forcing_chains(&board, false).expect("not found");
        assert_any_erase(got.actions(), cell!(H2), digit!(5));
    }

    #[test]
    fn figure_3_example() {
        let parser = Parse::packed_with_options(Options::errors());
        let (board, effects, failed) = parser.parse(
            "000060000007009501904370800300007258000000000785000003000082607206700980070090000",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_digit_forcing_chains(&board, false).expect("not found");
        assert!(got
            .actions()
            .iter()
            .any(|action| action.collect_sets().next().is_some()));
    }

    #[test]
    fn second_example_1() {
        let parser = Parse::packed_with_options(Options::errors());
        let (board, effects, failed) = parser.parse(
            "602000300000030650030010980004503000300902006000107830041370000753000000000000703",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_digit_forcing_chains(&board, false).expect("not found");
        assert_any_erase(got.actions(), cell!(D2), digit!(1));
        assert_any_erase(got.actions(), cell!(D8), digit!(1));
        assert_any_erase(got.actions(), cell!(D9), digit!(1));
    }

    #[test]
    fn second_example_2() {
        let parser = Parse::packed_with_options(Options::errors());
        let (board, effects, failed) = parser.parse(
            "602000300000030650030010980004503000300902006000107830041370000753000000000000703",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_digit_forcing_chains(&board, false).expect("not found");
        assert_any_erase(got.actions(), cell!(H7), digit!(2));
    }
}
