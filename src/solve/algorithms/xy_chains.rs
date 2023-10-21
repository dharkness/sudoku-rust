use super::*;

use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::rc::Rc;

pub fn find_xy_chains(board: &Board) -> Option<Effects> {
    let mut effects = Effects::new();

    let bi_values = board.cells_with_n_candidates(2);
    let mut forest = Forest::new();

    for cell in bi_values {
        forest.add_node(board, cell);
    }

    for k in Known::iter() {
        let candidates = board.candidate_cells(k);
        let mut found = Found::new(k);

        for graph in forest.graphs.values() {
            if graph.nodes.len() < 4 {
                continue;
            }

            let erasables = candidates & graph.peers[k.usize()];
            if erasables.is_empty() {
                continue;
            }

            let starts = erasables.iter().fold(CellSet::empty(), |acc, cell| {
                acc | (cell.peers() & candidates & graph.cells[k.usize()])
            });
            for start in starts {
                // find all chains from start
                let mut chains: VecDeque<Rc<Chain>> = VecDeque::new();
                chains.push_back(Rc::new(Chain::new(&graph.nodes[&start], k)));

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

        found.resolve(&mut effects)
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
            .union();

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
        peers[node.min_known.usize()] = root.peers();
        peers[node.max_known.usize()] = root.peers();

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
        self.peers[node.min_known.usize()].has(node.cell)
            || self.peers[node.max_known.usize()].has(node.cell)
    }

    fn add_node(&mut self, node: &Rc<Node>) {
        let cell = node.cell;
        let min_k = node.min_known.usize();
        let max_k = node.max_known.usize();

        self.cells[0].add(cell);
        self.cells[min_k].add(cell);
        self.cells[max_k].add(cell);

        let peers = node.cell.peers();
        self.peers[min_k].union_with(peers);
        self.peers[max_k].union_with(peers);

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
    pair: KnownSet,
    min_known: Known,
    min_edges: CellSet,
    max_known: Known,
    max_edges: CellSet,
}

impl Node {
    fn new(board: &Board, cell: Cell) -> Self {
        let edges = cell.peers() & board.cells_with_n_candidates(2);
        let pair = board.candidates(cell);
        let (min_known, max_known) = pair.as_pair().unwrap();

        Node {
            cell,
            pair,
            min_known,
            min_edges: (edges & board.candidate_cells(min_known)) - cell,
            max_known,
            max_edges: (edges & board.candidate_cells(max_known)) - cell,
        }
    }

    fn other(&self, known: Known) -> Known {
        if known == self.min_known {
            self.max_known
        } else if known == self.max_known {
            self.min_known
        } else {
            panic!("Node::other: known not in pair")
        }
    }

    fn edges(&self, known: Known) -> CellSet {
        if known == self.min_known {
            self.min_edges
        } else if known == self.max_known {
            self.max_edges
        } else {
            panic!("Node::other: known not in pair")
        }
    }
}

/// One chain is created per unique path in a graph and starting known.
/// They are extended with nodes along edges, and their links are shared when branching.
struct Chain {
    head: Rc<Link>,
    len: usize,
    start: Cell,
    start_known: Known,
    end: Cell,
    end_known: Known,
    visited: CellSet,
    erases: CellSet,
}

impl Chain {
    fn new(start: &Rc<Node>, known: Known) -> Self {
        let link = Rc::new(Link::new(start, known));
        let end_known = link.known;
        Chain {
            head: link,
            len: 1,
            start: start.cell,
            start_known: known,
            end: start.cell,
            end_known,
            visited: CellSet::empty() + start.cell,
            erases: CellSet::empty(),
        }
    }

    fn extend(&self, node: &Rc<Node>, erasable: CellSet) -> Rc<Self> {
        let head = Link::extend(&self.head, node);
        let len = head.len;
        let end_known = head.known;
        let erases = if len >= 4 && end_known == self.start_known {
            erasable
        } else {
            CellSet::empty()
        };

        Rc::new(Chain {
            head,
            len,
            start: self.start,
            start_known: self.start_known,
            end: node.cell,
            end_known,
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
            write!(f, "{} {} ", link.known, link.node.cell)?;
            link = tail;
        }
        write!(f, "{} {} {}", link.known, link.node.cell, link.tail_known)
    }
}

/// The links form the chain of nodes from the current end back to the starting cell.
/// They are shared among chains when a chain branches to multiple edges.
struct Link {
    tail: Option<Rc<Link>>,
    tail_known: Known,
    len: usize,
    node: Rc<Node>,
    known: Known,
}

impl Link {
    fn new(start: &Rc<Node>, known: Known) -> Self {
        Link {
            tail: None,
            tail_known: known,
            len: 1,
            node: Rc::clone(start),
            known: start.other(known),
        }
    }

    fn extend(tail: &Rc<Link>, node: &Rc<Node>) -> Rc<Self> {
        Rc::new(Link {
            tail: Some(Rc::clone(tail)),
            tail_known: tail.known,
            len: tail.len + 1,
            node: Rc::clone(node),
            known: node.other(tail.known),
        })
    }

    fn edges(&self) -> CellSet {
        self.node.edges(self.known)
    }
}

/// Tracks the shortest unique chains for a given starting known
/// and resolves them to the final set after searching all graphs for it.
struct Found {
    known: Known,
    erases: CellSet,
    chains: Vec<Rc<Chain>>,
}

impl Found {
    fn new(known: Known) -> Self {
        Found {
            known,
            erases: CellSet::empty(),
            chains: Vec::new(),
        }
    }

    fn add(&mut self, chain: &Rc<Chain>) {
        self.erases |= chain.erases;
        add_candidate(chain, &mut self.chains);
    }

    fn resolve(&self, effects: &mut Effects) {
        let mut remaining = self.erases;
        for chain in self.chains.iter().sorted_by(|left, right| {
            left.len
                .cmp(&right.len)
                .then(left.erases.len().cmp(&right.erases.len()))
        }) {
            effects.add_erase_cells(Strategy::XYChain, chain.erases, chain.start_known);

            remaining -= chain.erases;
            if remaining.is_empty() {
                break;
            }
        }
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
