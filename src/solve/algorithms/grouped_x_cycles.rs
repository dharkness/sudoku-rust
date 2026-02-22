use std::collections::{HashMap, VecDeque};

use super::*;

pub fn find_grouped_x_cycles(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    for digit in Digit::iter() {
        let graph = Graph::new(board, digit);
        if graph.nodes.len() < 4 {
            continue;
        }

        if add_rule_1(digit, &graph, single, &mut effects) {
            return Some(effects);
        }
        if add_rule_2(digit, &graph, single, &mut effects) {
            return Some(effects);
        }
        if add_rule_3(digit, &graph, single, &mut effects) {
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
    node: usize,
    next: Link,
}

#[derive(Clone, Debug)]
struct Node {
    cells: CellSet,
    houses: [House; 3],
    house_count: usize,
    grouped: bool,
}

impl Node {
    fn new_cell(cell: Cell) -> Self {
        let houses = cell.houses();
        Node {
            cells: CellSet::empty() + cell,
            houses,
            house_count: houses.len(),
            grouped: false,
        }
    }

    fn new_group(cells: CellSet, line: House, block: House) -> Self {
        Node {
            cells,
            houses: [line, block, block],
            house_count: 2,
            grouped: true,
        }
    }

    fn houses(&self) -> impl Iterator<Item = House> + '_ {
        self.houses[..self.house_count].iter().copied()
    }

    fn overlaps(&self, other: &Node) -> bool {
        self.cells.has_any(other.cells)
    }

    fn is_single(&self) -> bool {
        !self.grouped && self.cells.len() == 1
    }
}

struct HouseNodes {
    candidates: CellSet,
    nodes: Vec<usize>,
}

struct Graph {
    nodes: Vec<Node>,
    strong_adj: Vec<Vec<usize>>,
    weak_adj: Vec<Vec<usize>>,
    weak_houses: Vec<HouseNodes>,
}

impl Graph {
    fn new(board: &Board, digit: Digit) -> Self {
        let mut nodes = Vec::new();
        let mut known: HashMap<CellSet, usize> = HashMap::new();

        for cell in board.candidate_cells(digit) {
            add_node(&mut nodes, &mut known, Node::new_cell(cell));
        }

        for block in House::blocks_iter() {
            let block_candidates = board.house_candidate_cells(block, digit);
            if block_candidates.len() < 2 {
                continue;
            }

            for row in block.row_iter() {
                let segment = block.intersect(row);
                let segment_candidates = block_candidates & segment;
                if (2..=3).contains(&segment_candidates.len()) {
                    add_node(
                        &mut nodes,
                        &mut known,
                        Node::new_group(segment_candidates, row, block),
                    );
                }
            }

            for column in block.column_iter() {
                let segment = block.intersect(column);
                let segment_candidates = block_candidates & segment;
                if (2..=3).contains(&segment_candidates.len()) {
                    add_node(
                        &mut nodes,
                        &mut known,
                        Node::new_group(segment_candidates, column, block),
                    );
                }
            }
        }

        let mut nodes_in_house = vec![Vec::new(); house_count()];
        for (index, node) in nodes.iter().enumerate() {
            for house in node.houses() {
                nodes_in_house[house_index(house)].push(index);
            }
        }

        let mut strong_adj = vec![Vec::new(); nodes.len()];
        let mut weak_adj = vec![Vec::new(); nodes.len()];
        let mut weak_houses = Vec::new();

        for house in House::iter() {
            let candidates = board.house_candidate_cells(house, digit);
            if candidates.len() < 2 {
                continue;
            }

            let house_nodes = &nodes_in_house[house_index(house)];
            if house_nodes.len() < 2 {
                continue;
            }

            for i in 0..house_nodes.len() {
                for j in (i + 1)..house_nodes.len() {
                    let a = house_nodes[i];
                    let b = house_nodes[j];

                    if !nodes[a].overlaps(&nodes[b]) {
                        weak_adj[a].push(b);
                        weak_adj[b].push(a);
                    }

                    let union = nodes[a].cells | nodes[b].cells;
                    if union == candidates
                        && nodes[a].cells != candidates
                        && nodes[b].cells != candidates
                    {
                        strong_adj[a].push(b);
                        strong_adj[b].push(a);
                    }
                }
            }

            if candidates.len() > 2 {
                weak_houses.push(HouseNodes {
                    candidates,
                    nodes: house_nodes.clone(),
                });
            }
        }

        for index in 0..nodes.len() {
            strong_adj[index].sort_unstable();
            strong_adj[index].dedup();
            weak_adj[index].sort_unstable();
            weak_adj[index].dedup();
        }

        Graph {
            nodes,
            strong_adj,
            weak_adj,
            weak_houses,
        }
    }

    fn neighbors(&self, node: usize, link: Link) -> &[usize] {
        match link {
            Link::Strong => &self.strong_adj[node],
            Link::Weak => &self.weak_adj[node],
        }
    }

    fn is_strong_link(&self, a: usize, b: usize) -> bool {
        self.strong_adj[a].binary_search(&b).is_ok()
    }
}

fn add_node(nodes: &mut Vec<Node>, known: &mut HashMap<CellSet, usize>, node: Node) {
    if known.contains_key(&node.cells) {
        return;
    }
    let index = nodes.len();
    nodes.push(node);
    known.insert(nodes[index].cells, index);
}

fn house_count() -> usize {
    (House::COUNT as usize) * 3
}

fn house_index(house: House) -> usize {
    house.shape().usize() * House::COUNT as usize + house.usize()
}

fn add_rule_1(digit: Digit, graph: &Graph, single: bool, effects: &mut Effects) -> bool {
    for house in &graph.weak_houses {
        let nodes = &house.nodes;
        for i in 0..nodes.len() {
            for j in (i + 1)..nodes.len() {
                let a = nodes[i];
                let b = nodes[j];
                if graph.nodes[a].overlaps(&graph.nodes[b]) {
                    continue;
                }

                let Some(path) =
                    find_alternating_path(graph, a, b, Link::Strong, Link::Strong, None)
                else {
                    continue;
                };

                if path.len() < 4 {
                    continue;
                }

                let union = graph.nodes[a].cells | graph.nodes[b].cells;
                let erase = house.candidates - union;
                if erase.is_empty() {
                    continue;
                }

                let mut action = Action::new(Strategy::GroupedXCycle);
                action.erase_cells(erase, digit);
                add_chain_clues(&mut action, digit, &graph.nodes, &path);

                if effects.add_action(action) && single {
                    return true;
                }
            }
        }
    }

    false
}

fn add_rule_2(digit: Digit, graph: &Graph, single: bool, effects: &mut Effects) -> bool {
    for (node_index, node) in graph.nodes.iter().enumerate() {
        if !node.is_single() {
            continue;
        }

        let strong_neighbors = &graph.strong_adj[node_index];
        if strong_neighbors.len() < 2 {
            continue;
        }

        for i in 0..strong_neighbors.len() {
            for j in (i + 1)..strong_neighbors.len() {
                let a = strong_neighbors[i];
                let c = strong_neighbors[j];
                let Some(path) =
                    find_alternating_path(graph, a, c, Link::Weak, Link::Weak, Some(node_index))
                else {
                    continue;
                };

                let Some(cell) = node.cells.as_single() else {
                    continue;
                };
                let mut action = Action::new_set(Strategy::GroupedXCycle, cell, digit);
                add_chain_clues(&mut action, digit, &graph.nodes, &path);
                action.clue_cell_for_digit(Verdict::Primary, cell, digit);

                if effects.add_action(action) && single {
                    return true;
                }
            }
        }
    }

    false
}

fn add_rule_3(digit: Digit, graph: &Graph, single: bool, effects: &mut Effects) -> bool {
    for (node_index, node) in graph.nodes.iter().enumerate() {
        if !node.is_single() {
            continue;
        }

        let weak_neighbors = &graph.weak_adj[node_index];
        if weak_neighbors.len() < 2 {
            continue;
        }

        for i in 0..weak_neighbors.len() {
            for j in (i + 1)..weak_neighbors.len() {
                let a = weak_neighbors[i];
                let c = weak_neighbors[j];
                if graph.is_strong_link(node_index, a) && graph.is_strong_link(node_index, c) {
                    continue;
                }

                let Some(path) = find_alternating_path(
                    graph,
                    a,
                    c,
                    Link::Strong,
                    Link::Strong,
                    Some(node_index),
                ) else {
                    continue;
                };

                let Some(cell) = node.cells.as_single() else {
                    continue;
                };
                let mut action = Action::new_erase(Strategy::GroupedXCycle, cell, digit);
                add_chain_clues(&mut action, digit, &graph.nodes, &path);
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
    graph: &Graph,
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

    let mut visited = vec![[false; 2]; graph.nodes.len()];
    let mut prev: Vec<[Option<PrevState>; 2]> = vec![[None; 2]; graph.nodes.len()];
    let mut queue: VecDeque<(usize, Link)> = VecDeque::new();

    let start_idx = link_index(start_edge);
    visited[start][start_idx] = true;
    queue.push_back((start, start_edge));

    while let Some((node, next_link)) = queue.pop_front() {
        for &neighbor in graph.neighbors(node, next_link) {
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

fn add_chain_clues(action: &mut Action, digit: Digit, nodes: &[Node], path: &[usize]) {
    if path.is_empty() {
        return;
    }

    let mut verdict = Verdict::Primary;
    for &node_index in path {
        for cell in nodes[node_index].cells {
            action.clue_cell_for_digit(verdict, cell, digit);
        }
        verdict = match verdict {
            Verdict::Primary => Verdict::Secondary,
            _ => Verdict::Primary,
        };
    }
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
    fn rule_1_figure_2_example() {
        let parser = Parse::packed_with_options(Options::errors());
        let (board, effects, failed) = parser.parse(
            "185026370060000000097008100010052090000060000030179040041600950000000000056290700",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_grouped_x_cycles(&board, false).expect("not found");
        let actions = got.actions();
        assert_any_erase(actions, cell!(C9), digit!(4));
        assert_any_erase(actions, cell!(H9), digit!(4));
        assert_any_erase(actions, cell!(H6), digit!(4));
        assert_any_erase(actions, cell!(C4), digit!(4));
    }

    #[test]
    fn rule_1_figure_3_example() {
        let parser = Parse::packed_with_options(Options::errors());
        let (board, effects, failed) = parser.parse(
            "001004060804000010000703000030002009000090000600100040000506000050000401040200700",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_grouped_x_cycles(&board, false).expect("not found");
        assert!(got.action_count() > 0);
    }

    #[test]
    fn rule_2_figure_4_example() {
        let parser = Parse::packed_with_options(Options::errors());
        let (board, effects, failed) = parser.parse(
            "080007060700940008000380700070000100018050940003000080001065000400039006090800070",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_grouped_x_cycles(&board, false).expect("not found");
        assert!(got.action_count() > 0);
    }

    #[test]
    fn rule_2_figure_5_example() {
        let parser = Parse::packed_with_options(Options::errors());
        let (board, effects, failed) = parser.parse(
            "204007905090502040035498200006029504002040090049350802003974020920003457400205309",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_grouped_x_cycles(&board, false).expect("not found");
        assert_any_set(got.actions(), cell!(G9), digit!(8));
    }

    #[test]
    fn rule_3_figure_6_example() {
        let parser = Parse::packed_with_options(Options::errors());
        let (board, effects, failed) = parser.parse(
            "003706500000500001069040000002000050090602010070000200000050730400009000001307900",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_grouped_x_cycles(&board, false).expect("not found");
        assert!(got.action_count() > 0);
    }

    #[test]
    fn consecutive_grouped_cells_example() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "S9B015w0e2e480i04065w03040906445w6c0a6c505w50010d050cb8d04c0i4c4i0a565g07030506482e095u0a04440g0a4e025i1m5ebmci5805580i070a5003585844074q5g03094k01090c014q5g50784k7g",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_grouped_x_cycles(&board, false).expect("not found");
        assert_any_erase(got.actions(), cell!(C1), digit!(2));
    }

    #[test]
    fn grouped_cycle_with_eight_nodes_example() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "S9B0f02be4a050cbe070a030abe62d87wbg1u1u16bu0701bg06037w4a30bubg03be01068c6i2kbibk5mcabud801660aby0602be07be8a4u0s0u0109684c051m3e0807181u1p180r0309090f1a2y2f162j0802",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_grouped_x_cycles(&board, false).expect("not found");
        assert_any_erase(got.actions(), cell!(C5), digit!(4));
    }
}
