use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::rc::Rc;

use super::*;

pub fn find_xy_chains(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    let bi_values = board.cells_with_n_candidates(2);
    let mut forest = Forest::new();

    for cell in bi_values {
        forest.add_node(board, cell);
    }

    for d in Digit::iter() {
        let candidates = board.candidate_cells(d);
        let mut found = Found::new(d);

        for graph in forest.graphs.values() {
            if graph.nodes.len() < 4 {
                continue;
            }

            let erasables = candidates & graph.peers[d.usize()];
            if erasables.is_empty() {
                continue;
            }

            let starts = erasables.iter().fold(CellSet::empty(), |acc, cell| {
                acc | (cell.peers() & candidates & graph.cells[d.usize()])
            });
            for start in starts {
                // find all chains from start
                let mut chains: VecDeque<Rc<Chain>> = VecDeque::new();
                chains.push_back(Rc::new(Chain::new(&graph.nodes[&start], d)));

                while let Some(chain) = chains.pop_front() {
                    for end in chain.edges() {
                        let erasable = start.peers() & end.peers() & erasables;
                        let extended = Chain::extend(&chain, &graph.nodes[&end], erasable);

                        if !extended.erases.is_empty() {
                            found.add(&extended);
                        }
                        if !extended.edges().is_empty() {
                            chains.push_back(extended);
                        }
                    }
                }
            }
        }

        if found.resolve(single, &mut effects) {
            return Some(effects);
        }
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

/// Builds graphs from cells with two candidates and merges them when they connect.
struct Forest {
    graphs: HashMap<Cell, Graph>,
}

impl Forest {
    fn new() -> Self {
        Forest {
            graphs: HashMap::new(),
        }
    }

    fn add_node(&mut self, board: &Board, cell: Cell) {
        let node = Rc::new(Node::new(board, cell));

        let mut sees = self
            .graphs
            .iter()
            .filter(|(_, g)| g.can_add_node(&node))
            .map(|(c, _)| *c)
            .union_cells();

        if sees.is_empty() {
            self.graphs.insert(cell, Graph::new(&node));
        } else if sees.len() == 1 {
            let root = sees.pop().unwrap();
            self.graphs.get_mut(&root).unwrap().add_node(&node);
        } else {
            let root = sees.pop().unwrap();
            let mut graph = self.graphs.remove(&root).unwrap();
            graph.add_node(&node);

            for seen in sees {
                graph.merge(self.graphs.remove(&seen).unwrap());
            }

            self.graphs.insert(root, graph);
        }
    }
}

/// Holds all connected peer cells in a cyclic graph.
struct Graph {
    root: Cell,
    cells: [CellSet; 9],
    peers: [CellSet; 9],
    nodes: HashMap<Cell, Rc<Node>>,
}

impl Graph {
    fn new(node: &Rc<Node>) -> Self {
        let root = node.cell;
        let mut cells = [CellSet::empty(); 9];
        cells[0] = CellSet::of(&[root]);

        let mut peers = [CellSet::empty(); 9];
        peers[node.min_digit.usize()] = root.peers();
        peers[node.max_digit.usize()] = root.peers();

        let mut nodes = HashMap::new();
        nodes.insert(root, Rc::clone(node));

        Graph {
            root,
            cells,
            peers,
            nodes,
        }
    }

    fn can_add_node(&self, node: &Rc<Node>) -> bool {
        self.peers[node.min_digit.usize()].has(node.cell)
            || self.peers[node.max_digit.usize()].has(node.cell)
    }

    fn add_node(&mut self, node: &Rc<Node>) {
        let cell = node.cell;
        let min_d = node.min_digit.usize();
        let max_d = node.max_digit.usize();

        self.cells[0].add(cell);
        self.cells[min_d].add(cell);
        self.cells[max_d].add(cell);

        let peers = node.cell.peers();
        self.peers[min_d].union_with(peers);
        self.peers[max_d].union_with(peers);

        self.nodes.insert(cell, Rc::clone(node));
    }

    fn merge(&mut self, other: Graph) {
        self.cells
            .iter_mut()
            .enumerate()
            .for_each(|(i, set)| set.union_with(other.cells[i]));

        for (i, peers) in other.peers.iter().enumerate() {
            self.peers[i].union_with(*peers);
        }

        self.nodes.extend(other.nodes);
    }
}

/// One node is created for each cell with two candidates and shared among all graphs.
struct Node {
    cell: Cell,
    pair: DigitSet,
    min_digit: Digit,
    min_edges: CellSet,
    max_digit: Digit,
    max_edges: CellSet,
}

impl Node {
    fn new(board: &Board, cell: Cell) -> Self {
        let edges = cell.peers() & board.cells_with_n_candidates(2);
        let pair = board.candidates(cell);
        let (min_digit, max_digit) = pair.as_pair().unwrap();

        Node {
            cell,
            pair,
            min_digit,
            min_edges: (edges & board.candidate_cells(min_digit)) - cell,
            max_digit,
            max_edges: (edges & board.candidate_cells(max_digit)) - cell,
        }
    }

    fn other(&self, digit: Digit) -> Digit {
        if digit == self.min_digit {
            self.max_digit
        } else if digit == self.max_digit {
            self.min_digit
        } else {
            panic!(
                "digit {} not in pair [{}, {}]",
                digit, self.min_digit, self.max_digit
            )
        }
    }

    fn edges(&self, digit: Digit) -> CellSet {
        if digit == self.min_digit {
            self.min_edges
        } else if digit == self.max_digit {
            self.max_edges
        } else {
            panic!(
                "digit {} not in pair [{}, {}]",
                digit, self.min_digit, self.max_digit
            )
        }
    }
}

/// One chain is created per unique path in a graph and starting digit.
/// They are extended with nodes along edges, and their links are shared when branching.
struct Chain {
    head: Rc<Link>,
    len: usize,
    start: Cell,
    start_digit: Digit,
    end: Cell,
    end_digit: Digit,
    visited: CellSet,
    erases: CellSet,
}

impl Chain {
    fn new(start: &Rc<Node>, digit: Digit) -> Self {
        let link = Rc::new(Link::new(start, digit));
        let end_digit = link.digit;
        Chain {
            head: link,
            len: 1,
            start: start.cell,
            start_digit: digit,
            end: start.cell,
            end_digit,
            visited: CellSet::empty() + start.cell,
            erases: CellSet::empty(),
        }
    }

    fn extend(&self, node: &Rc<Node>, erasable: CellSet) -> Rc<Self> {
        let head = Link::extend(&self.head, node);
        let len = head.len;
        let end_digit = head.digit;
        let erases = if len >= 4 && end_digit == self.start_digit {
            erasable
        } else {
            CellSet::empty()
        };

        Rc::new(Chain {
            head,
            len,
            start: self.start,
            start_digit: self.start_digit,
            end: node.cell,
            end_digit,
            visited: self.visited + node.cell,
            erases,
        })
    }

    fn edges(&self) -> CellSet {
        self.head.edges() - self.visited
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut link = &self.head;
        while let Some(tail) = &link.tail {
            write!(f, "{} {} ", link.digit, link.node.cell)?;
            link = tail;
        }
        write!(f, "{} {} {}", link.digit, link.node.cell, link.tail_digit)
    }
}

/// The links form the chain of nodes from the current end back to the starting cell.
/// They are shared among chains when a chain branches to multiple edges.
struct Link {
    tail: Option<Rc<Link>>,
    tail_digit: Digit,
    len: usize,
    node: Rc<Node>,
    digit: Digit,
}

impl Link {
    fn new(start: &Rc<Node>, digit: Digit) -> Self {
        Link {
            tail: None,
            tail_digit: digit,
            len: 1,
            node: Rc::clone(start),
            digit: start.other(digit),
        }
    }

    fn extend(tail: &Rc<Link>, node: &Rc<Node>) -> Rc<Self> {
        Rc::new(Link {
            tail: Some(Rc::clone(tail)),
            tail_digit: tail.digit,
            len: tail.len + 1,
            node: Rc::clone(node),
            digit: node.other(tail.digit),
        })
    }

    fn edges(&self) -> CellSet {
        self.node.edges(self.digit)
    }
}

/// Tracks the shortest unique chains for a given starting digit
/// and resolves them to the final set after searching all graphs for it.
struct Found {
    digit: Digit,
    erases: CellSet,
    chains: Vec<Rc<Chain>>,
}

impl Found {
    fn new(digit: Digit) -> Self {
        Found {
            digit,
            erases: CellSet::empty(),
            chains: Vec::new(),
        }
    }

    fn add(&mut self, chain: &Rc<Chain>) {
        self.erases |= chain.erases;
        add_candidate(chain, &mut self.chains);
    }

    fn resolve(&self, single: bool, effects: &mut Effects) -> bool {
        let mut remaining = self.erases;
        for chain in self.chains.iter().sorted_by(|left, right| {
            left.len
                .cmp(&right.len)
                .then(left.erases.len().cmp(&right.erases.len()))
        }) {
            let mut action =
                Action::new_erase_cells(Strategy::XYChain, chain.erases, chain.start_digit);
            let mut link = Some(&chain.head);
            while let Some(next) = link {
                let cell = next.node.cell;
                let digit = next.node.other(next.digit);
                action.clue_cell_for_digit(Verdict::Secondary, cell, digit);
                action.clue_cell_for_digit(Verdict::Tertiary, cell, next.digit);
                link = next.tail.as_ref();
            }

            if effects.add_action(action) && single {
                return true;
            }

            remaining -= chain.erases;
            if remaining.is_empty() {
                break;
            }
        }

        false
    }
}

/// Removes any chains that are the same length or longer without erasing additional cells
/// and adds the new chain unless there is a shorter one that erases the same cells.
fn add_candidate(new: &Rc<Chain>, chains: &mut Vec<Rc<Chain>>) {
    let mut remove: Vec<usize> = Vec::new();
    let mut add = true;

    for (i, chain) in chains.iter().enumerate() {
        if new.len < chain.len {
            if new.erases.has_all(chain.erases) {
                remove.push(i);
            }
        } else if chain.erases.has_all(new.erases) {
            add = false;
            break;
        }
    }

    for i in remove.iter().rev() {
        chains.remove(*i);
    }

    if add {
        chains.push(Rc::clone(new));
    }
}

#[cfg(test)]
mod tests {
    use crate::io::{Parse, Parser};
    use crate::*;

    use super::*;

    #[test]
    fn test() {
        let parser = Parse::wiki().stop_on_error();
        let (board, ..) = parser.parse(
            "441181i402i4k4080h0g20g10884418411024c0c03o4100gs421g4p4o4410h09q403o030o6om091184o42go040p0og20o040031g0508g2g214a40ha409403020411403g108100g8188880g412011g402g4",
        );

        if let Some(got) = find_xy_chains(&board, true) {
            let mut action = Action::new(Strategy::XYChain);
            action.erase(cell!(C4), digit!(9));
            action.clue_cells_for_digit(Verdict::Secondary, cells![B7 E5], digit!(2));
            action.clue_cells_for_digit(Verdict::Tertiary, cells![B5 C9], digit!(2));
            action.clue_cells_for_digit(Verdict::Secondary, cells![B5 F4], digit!(8));
            action.clue_cells_for_digit(Verdict::Tertiary, cells![B7 E5], digit!(8));
            action.clue_cells_for_digit(Verdict::Secondary, cells![C9], digit!(9));
            action.clue_cells_for_digit(Verdict::Tertiary, cells![F4], digit!(9));

            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("no effects found");
        }
    }
}
