use std::collections::VecDeque;

use super::*;

pub fn find_x_cycles(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    for digit in Digit::iter() {
        let candidates = board.candidate_cells(digit);
        if candidates.len() < 4 {
            continue;
        }

        let links = Links::new(board, digit);

        if add_discontinuous_strong(board, digit, &links, single, &mut effects) {
            return Some(effects);
        }

        if add_discontinuous_weak(board, digit, &links, single, &mut effects) {
            return Some(effects);
        }

        if add_continuous_loops(board, digit, &links, single, &mut effects) {
            return Some(effects);
        }
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Link {
    Strong,
    Weak,
}

impl Link {
    fn toggle(self) -> Self {
        match self {
            Self::Strong => Self::Weak,
            Self::Weak => Self::Strong,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct PrevState {
    cell: Cell,
    next: Link,
}

struct Links {
    strong_adj: [CellSet; Cell::COUNT as usize],
    weak_adj: [CellSet; Cell::COUNT as usize],
    weak_houses: Vec<CellSet>,
}

impl Links {
    fn new(board: &Board, digit: Digit) -> Self {
        let mut strong_adj = [CellSet::empty(); Cell::COUNT as usize];
        let mut weak_adj = [CellSet::empty(); Cell::COUNT as usize];
        let mut weak_houses = Vec::new();

        for house in House::iter() {
            let cells = board.house_candidate_cells(house, digit);
            if cells.len() < 2 {
                continue;
            }

            if cells.len() == 2 {
                let (a, b) = cells.as_pair().unwrap();
                strong_adj[a.usize()].add(b);
                strong_adj[b.usize()].add(a);
            } else {
                weak_houses.push(cells);
            }

            for cell in cells {
                weak_adj[cell.usize()].union_with(cells - cell);
            }
        }

        Links {
            strong_adj,
            weak_adj,
            weak_houses,
        }
    }

    fn neighbors(&self, cell: Cell, link: Link) -> CellSet {
        match link {
            Link::Strong => self.strong_adj[cell.usize()],
            Link::Weak => self.weak_adj[cell.usize()],
        }
    }

    fn is_strong_link(&self, a: Cell, b: Cell) -> bool {
        self.strong_adj[a.usize()].has(b)
    }
}

fn add_continuous_loops(
    _board: &Board,
    digit: Digit,
    links: &Links,
    single: bool,
    effects: &mut Effects,
) -> bool {
    for house_cells in &links.weak_houses {
        for (a, b) in house_cells.pair_iter() {
            let Some(path) = find_alternating_path(links, a, b, Link::Strong, Link::Strong, None)
            else {
                continue;
            };

            if path.len() < 4 {
                continue;
            }

            let erase = (*house_cells - a) - b;
            if erase.is_empty() {
                continue;
            }

            let mut action = Action::new_erase_cells(Strategy::XCycle, erase, digit);
            add_chain_clues(&mut action, digit, &path, Verdict::Primary);

            if effects.add_action(action) && single {
                return true;
            }
        }
    }

    false
}

fn add_discontinuous_strong(
    board: &Board,
    digit: Digit,
    links: &Links,
    single: bool,
    effects: &mut Effects,
) -> bool {
    for b in board.candidate_cells(digit) {
        let strong_neighbors = links.strong_adj[b.usize()];
        if strong_neighbors.len() < 2 {
            continue;
        }

        for (a, c) in strong_neighbors.pair_iter() {
            let Some(path) = find_alternating_path(links, a, c, Link::Weak, Link::Weak, Some(b))
            else {
                continue;
            };

            let mut action = Action::new_set(Strategy::XCycle, b, digit);
            add_chain_clues(&mut action, digit, &path, Verdict::Primary);
            action.clue_cell_for_digit(Verdict::Primary, b, digit);

            if effects.add_action(action) && single {
                return true;
            }
        }
    }

    false
}

fn add_discontinuous_weak(
    board: &Board,
    digit: Digit,
    links: &Links,
    single: bool,
    effects: &mut Effects,
) -> bool {
    for b in board.candidate_cells(digit) {
        let weak_neighbors = links.weak_adj[b.usize()];
        if weak_neighbors.len() < 2 {
            continue;
        }

        for (a, c) in weak_neighbors.pair_iter() {
            if links.is_strong_link(b, a) && links.is_strong_link(b, c) {
                continue;
            }

            let Some(path) =
                find_alternating_path(links, a, c, Link::Strong, Link::Strong, Some(b))
            else {
                continue;
            };

            let mut action = Action::new_erase(Strategy::XCycle, b, digit);
            add_chain_clues(&mut action, digit, &path, Verdict::Primary);
            action.clue_cell_for_digit(Verdict::Primary, b, digit);

            if effects.add_action(action) && single {
                return true;
            }
        }
    }

    false
}

fn find_alternating_path(
    links: &Links,
    start: Cell,
    end: Cell,
    start_edge: Link,
    end_edge: Link,
    banned: Option<Cell>,
) -> Option<Vec<Cell>> {
    if start == end {
        return None;
    }
    if banned == Some(start) || banned == Some(end) {
        return None;
    }

    let mut visited = [[false; 2]; Cell::COUNT as usize];
    let mut prev: [[Option<PrevState>; 2]; Cell::COUNT as usize] =
        [[None; 2]; Cell::COUNT as usize];
    let mut queue: VecDeque<(Cell, Link)> = VecDeque::new();

    let start_idx = link_index(start_edge);
    visited[start.usize()][start_idx] = true;
    queue.push_back((start, start_edge));

    while let Some((cell, next_link)) = queue.pop_front() {
        let neighbors = links.neighbors(cell, next_link);
        for neighbor in neighbors {
            if banned == Some(neighbor) {
                continue;
            }

            let next_state = next_link.toggle();
            let next_idx = link_index(next_state);
            if visited[neighbor.usize()][next_idx] {
                continue;
            }

            visited[neighbor.usize()][next_idx] = true;
            prev[neighbor.usize()][next_idx] = Some(PrevState {
                cell,
                next: next_link,
            });

            if neighbor == end && next_link == end_edge {
                return Some(reconstruct_path(start, end, next_state, &prev));
            }

            queue.push_back((neighbor, next_state));
        }
    }

    None
}

fn reconstruct_path(
    start: Cell,
    end: Cell,
    end_state: Link,
    prev: &[[Option<PrevState>; 2]],
) -> Vec<Cell> {
    let mut path: Vec<Cell> = Vec::new();
    let mut cell = end;
    let mut state = end_state;

    loop {
        path.push(cell);
        if cell == start {
            break;
        }
        let index = link_index(state);
        let Some(prev_state) = prev[cell.usize()][index] else {
            break;
        };
        cell = prev_state.cell;
        state = prev_state.next;
    }

    path.reverse();
    path
}

fn link_index(link: Link) -> usize {
    match link {
        Link::Strong => 0,
        Link::Weak => 1,
    }
}

fn add_chain_clues(action: &mut Action, digit: Digit, path: &[Cell], start: Verdict) {
    if path.is_empty() {
        return;
    }

    let mut verdict = start;
    for cell in path {
        action.clue_cell_for_digit(verdict, *cell, digit);
        verdict = match verdict {
            Verdict::Primary => Verdict::Secondary,
            _ => Verdict::Primary,
        };
    }
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
    fn part1_figure_4_example() {
        let parser = Parse::packed_with_options(Options::all());
        let (board, effects, failed) = parser.parse(
            "024100670060070410700964020246591387135487296879623154400009760350716940697040031",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_x_cycles(&board, false).expect("not found");
        let actions = got.actions();
        assert_any_erase(actions, cell!(C3), digit!(8));
        assert_any_erase(actions, cell!(C9), digit!(8));
        assert_any_erase(actions, cell!(G3), digit!(8));
        assert_any_erase(actions, cell!(G9), digit!(8));
    }

    #[test]
    fn part1_cool_off_chain_example() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "S9B014m82dad27y029u0fdw5g90cyd17p039v049k047o059l06cy9f5v1k071x8m93088204141k2008ae0486019k2w0u0917022v136a066e8s1g8s012007220814055107cec8801m0p0p585103565g183u2d09",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_x_cycles(&board, false).expect("not found");
        let actions = got.actions();
        assert_any_erase(actions, cell!(H9), digit!(3));
    }

    #[test]
    fn part2_rule_2_example() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "S9B0h7n04050c078j1h0l9e0b03060a0d9e080e062b0e0i080b2b0c047y1q8k018q0e0h077o057u0l077u080c0l069f08ab0b8i030d0e7n023e370h050i1f1n037u058i0c0g01028q0h7r7q0h040b06057n0g",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_x_cycles(&board, false).expect("not found");
        assert_any_set(got.actions(), cell!(J1), digit!(1));
    }

    #[test]
    fn part2_rule_3_example_1() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "S9B4j0g060212b604bmbb4i09040a120g4o0648020n4n040fb6bnbm07bm0f4k03070abobwbg0704460e090b460106830p150f0804880g7s030l0i0g0t06454c050f080l0i0t0e07030t0d0e070h0l03067o7p",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_x_cycles(&board, false).expect("not found");
        assert_any_erase(got.actions(), cell!(C3), digit!(1));
    }

    #[test]
    fn part2_rule_3_example_2() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "S9B0z0g060212b604bmbb4i09040a120g480648020n4m040fb6b7bm07bm0f4k03070abobgbg0704460e090b460106830p150f0804880g7s030l0i0g0t06454c050f080l0i0t0e07030t0d0e070h0l03067o7p",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_x_cycles(&board, false).expect("not found");
        assert_any_erase(got.actions(), cell!(B7), digit!(8));
    }
}
