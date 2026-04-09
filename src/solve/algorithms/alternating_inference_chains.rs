use std::collections::VecDeque;

use super::*;

const DIGIT_COUNT: usize = Digit::COUNT as usize;
const CELL_COUNT: usize = Cell::COUNT as usize;
const NODE_COUNT: usize = DIGIT_COUNT * CELL_COUNT;

pub fn find_alternating_inference_chains(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();
    let links = Links::new(board);

    if add_rule_1(board, &links, single, &mut effects) {
        return Some(effects);
    }
    if add_rule_2(board, &links, single, &mut effects) {
        return Some(effects);
    }
    if add_rule_3(board, &links, single, &mut effects) {
        return Some(effects);
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
    node: usize,
    next: Link,
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

    fn neighbors(&self, node: usize, link: Link) -> &[usize] {
        match link {
            Link::Strong => &self.strong[node],
            Link::Weak => &self.weak[node],
        }
    }

    fn is_strong_link(&self, a: usize, b: usize) -> bool {
        self.strong[a].binary_search(&b).is_ok()
    }
}

fn add_rule_1(board: &Board, links: &Links, single: bool, effects: &mut Effects) -> bool {
    for cell in Cell::iter() {
        let digits = board.candidates(cell);
        if digits.len() < 2 {
            continue;
        }

        let digit_vec: Vec<Digit> = digits.iter().collect();
        for i in 0..digit_vec.len() {
            for j in (i + 1)..digit_vec.len() {
                let d1 = digit_vec[i];
                let d2 = digit_vec[j];
                let start = node_index(cell, d1);
                let end = node_index(cell, d2);

                let Some(path) =
                    find_alternating_path(links, start, end, Link::Strong, Link::Strong, None)
                else {
                    continue;
                };

                if path.len() < 4 {
                    continue;
                }

                let remove = digits - d1 - d2;
                if remove.is_empty() {
                    continue;
                }

                let mut action = Action::new(Strategy::AlternatingInferenceChain);
                action.erase_digits(cell, remove);
                add_chain_clues(&mut action, &path);

                if effects.add_action(action) && single {
                    return true;
                }
            }
        }
    }

    for digit in Digit::iter() {
        for house in House::iter() {
            let cells = board.house_candidate_cells(house, digit);
            if cells.len() < 2 {
                continue;
            }

            for (first, second) in cells.pair_iter() {
                let start = node_index(first, digit);
                let end = node_index(second, digit);

                let Some(path) =
                    find_alternating_path(links, start, end, Link::Strong, Link::Strong, None)
                else {
                    continue;
                };

                if path.len() < 4 {
                    continue;
                }

                let remove = (cells - first) - second;
                if remove.is_empty() {
                    continue;
                }

                let mut action = Action::new(Strategy::AlternatingInferenceChain);
                action.erase_cells(remove, digit);
                add_chain_clues(&mut action, &path);

                if effects.add_action(action) && single {
                    return true;
                }
            }
        }
    }

    false
}

fn add_rule_2(_board: &Board, links: &Links, single: bool, effects: &mut Effects) -> bool {
    for node in 0..NODE_COUNT {
        if !links.present[node] {
            continue;
        }

        let neighbors = &links.strong[node];
        if neighbors.len() < 2 {
            continue;
        }

        for i in 0..neighbors.len() {
            for j in (i + 1)..neighbors.len() {
                let start = neighbors[i];
                let end = neighbors[j];
                let Some(path) =
                    find_alternating_path(links, start, end, Link::Weak, Link::Weak, Some(node))
                else {
                    continue;
                };

                let (cell, digit) = node_cell_digit(node);
                let mut action = Action::new_set(Strategy::AlternatingInferenceChain, cell, digit);
                add_chain_clues(&mut action, &path);
                action.clue_cell_for_digit(Verdict::Primary, cell, digit);

                if effects.add_action(action) && single {
                    return true;
                }
            }
        }
    }

    false
}

fn add_rule_3(_board: &Board, links: &Links, single: bool, effects: &mut Effects) -> bool {
    for node in 0..NODE_COUNT {
        if !links.present[node] {
            continue;
        }

        let neighbors = &links.weak[node];
        if neighbors.len() < 2 {
            continue;
        }

        for i in 0..neighbors.len() {
            for j in (i + 1)..neighbors.len() {
                let start = neighbors[i];
                let end = neighbors[j];
                if links.is_strong_link(node, start) || links.is_strong_link(node, end) {
                    continue;
                }

                let Some(path) = find_alternating_path(
                    links,
                    start,
                    end,
                    Link::Strong,
                    Link::Strong,
                    Some(node),
                ) else {
                    continue;
                };

                let (cell, digit) = node_cell_digit(node);
                let mut action =
                    Action::new_erase(Strategy::AlternatingInferenceChain, cell, digit);
                add_chain_clues(&mut action, &path);
                action.clue_cell_for_digit(Verdict::Primary, cell, digit);

                if effects.add_action(action) && single {
                    return true;
                }
            }
        }
    }

    false
}

fn find_alternating_path(
    links: &Links,
    start: usize,
    end: usize,
    start_edge: Link,
    end_edge: Link,
    banned: Option<usize>,
) -> Option<Vec<usize>> {
    if start == end {
        return None;
    }
    if banned == Some(start) || banned == Some(end) {
        return None;
    }

    let mut visited = [[false; 2]; NODE_COUNT];
    let mut prev: [[Option<PrevState>; 2]; NODE_COUNT] = [[None; 2]; NODE_COUNT];
    let mut queue: VecDeque<(usize, Link)> = VecDeque::new();

    let start_idx = link_index(start_edge);
    visited[start][start_idx] = true;
    queue.push_back((start, start_edge));

    while let Some((node, next_link)) = queue.pop_front() {
        for &neighbor in links.neighbors(node, next_link) {
            if banned == Some(neighbor) {
                continue;
            }

            let next_state = next_link.toggle();
            let next_idx = link_index(next_state);
            if visited[neighbor][next_idx] {
                continue;
            }

            visited[neighbor][next_idx] = true;
            prev[neighbor][next_idx] = Some(PrevState {
                node,
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
    start: usize,
    end: usize,
    end_state: Link,
    prev: &[[Option<PrevState>; 2]],
) -> Vec<usize> {
    let mut path = Vec::new();
    let mut node = end;
    let mut state = end_state;

    loop {
        path.push(node);
        if node == start {
            break;
        }
        let index = link_index(state);
        let Some(prev_state) = prev[node][index] else {
            break;
        };
        node = prev_state.node;
        state = prev_state.next;
    }

    path.reverse();
    path
}

fn add_chain_clues(action: &mut Action, path: &[usize]) {
    let mut verdict = Verdict::Primary;
    for &node in path {
        let (cell, digit) = node_cell_digit(node);
        action.clue_cell_for_digit(verdict, cell, digit);
        verdict = match verdict {
            Verdict::Primary => Verdict::Secondary,
            _ => Verdict::Primary,
        };
    }
}

fn add_edge(adj: &mut [Vec<usize>], a: usize, b: usize) {
    adj[a].push(b);
    adj[b].push(a);
}

fn node_index(cell: Cell, digit: Digit) -> usize {
    cell.usize() * DIGIT_COUNT + digit.usize()
}

fn node_cell_digit(node: usize) -> (Cell, Digit) {
    let cell = Cell::new((node / DIGIT_COUNT) as u8);
    let digit = Digit::new((node % DIGIT_COUNT) as u32);
    (cell, digit)
}

fn link_index(link: Link) -> usize {
    match link {
        Link::Strong => 0,
        Link::Weak => 1,
    }
}

#[cfg(test)]
mod tests {
    use crate::io::{Parse, Parser};
    use crate::*;

    use super::*;

    #[test]
    fn rule_1_example() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "S9B1v8j4i6a046b83030b020c074j0z09060d4j0z7v4q02030683074j090r0c4j0f4q020z071v3f162b0b2i0h0i0c082b020c0i2r0z0f040c050a0907020d0h0f0g0h06040z0z030b090d020i0f080c0g0z0z",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_alternating_inference_chains(&board, false).expect("not found");
        assert_erase!(got, A4, 8);
    }

    #[test]
    fn rule_1_off_chain_in_cells() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "S9B05be8q4d074c0t8s037q019q061a1c3008a25262024v4v092z223v0p061b6o4v6o091a2y078a8f1b9b260826027s8c08349a3232013u456c2r094u6m061c17c4038y4q5m0118078a049uar2u023q138608",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_alternating_inference_chains(&board, false).expect("not found");
        assert_erase!(got, H1, 8);
    }

    #[test]
    fn rule_2_example() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "S9B0504037n0h7n0b0g060bdudu0c2216014q82b60ac2072202be4q0c074o4i11090z4y52047y86010616080g7q0b06b8be0s030g05b6017uaaay080b038q0a82464602820a968q220g018y968a0g96030208",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_alternating_inference_chains(&board, false).expect("not found");
        assert_set!(got, B9, 9);
    }

    #[test]
    fn rule_3_example() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "S9Bbu4q060a039m2q02cyb6030a9e0e020604cy070b8a8i0h8q0c820102067u050g01087q0u832q080d0f0c029f9e0r2i03020i082j0605034q18cy0l9u2z2r06220907030l1u17080s5e01106q043m092u0o",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_alternating_inference_chains(&board, false).expect("not found");
        assert_erase!(got, A2, 5);
    }
}
