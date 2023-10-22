use std::collections::HashMap;

use super::*;

/// Builds graphs of strong links between cells to solve graphs and eliminate candidates.
///
/// - Rule 2 - 05k088880g02k010210gk81021c005s00248c0032048g110c40c0h0910c00204g1210gc0020h04c0204810g188c020g111880g440c0311800g0403200941g0g004480h4881022010214802g010480g8005
/// - Rule 4 - 05l8d8c80g02s018210gl8d821d805s002c8c00320c8g158c41c0h0950d00204g1210gc0020h04c8204810g1c8c020g111c80gc40c0311800g0403200941g0g004480h4881022010214802g058580g8005
///
/// https://www.sudokuwiki.org/Singles_Chains
pub fn find_simple_colorings(board: &Board) -> Option<Effects> {
    let mut effects = Effects::new();

    for known in Known::iter() {
        let possibles = board.candidate_cells(known);
        if possibles.is_empty() {
            continue;
        }

        let forest = board
            .house_candidates_with_n_candidate_cells(2, known)
            .fold(Forest::new(known), |mut forest, (house, edge)| {
                // ignore block edges that share a row/column
                if !(house.is_block() && (edge.share_row() || edge.share_column())) {
                    forest.add_edge(edge);
                }
                forest
            });
        if forest.is_empty() {
            continue;
        }

        // forest.dump();

        // simple coloring

        for graph in forest.graphs.values() {
            // rule 2 - two of one color in a house
            if let Some(color) = graph.has_same_color_nodes_in_one_house() {
                // remove all nodes of that color and solve all nodes of the other
                let mut action = Action::new(Strategy::SimpleColoring);

                action.erase_cells(graph.nodes_of_color(color), known);
                action.set_cells(graph.nodes_of_color(color.other()), known);
                effects.add_action(action);
            }

            // rule 4 - candidate sees two nodes of opposite colors
            let erase = possibles
                .iter()
                .filter(|c| graph.both_colors_see_cell(*c))
                .union() as CellSet;
            effects.add_erase_cells(Strategy::SimpleColoring, erase, known);
        }

        // multi-coloring

        // find graph pairs where rule 4 found a node of one
        // not just that candidate is erased but all nodes of that color in its graph
        // essentially, one graph uses rule 4 to trigger rule 2 in another
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

/// Builds graphs from cells with a shared known and merges them when they connect.
struct Forest {
    known: Known,
    // The key, the first cell in the graph, is arbitrary.
    graphs: HashMap<Cell, Graph>,
}

impl Forest {
    fn new(known: Known) -> Self {
        Forest {
            known,
            graphs: HashMap::new(),
        }
    }

    fn is_empty(&self) -> bool {
        self.graphs.is_empty()
    }

    fn add_edge(&mut self, edge: CellSet) {
        let mut found = self
            .graphs
            .iter()
            .filter(|(_, graph)| graph.has_either_node(edge))
            .map(|(root, _)| *root)
            .union() as CellSet;

        match found.len() {
            0 => self.add_graph(edge),
            1 => self.add_graph_edge(found.pop().unwrap(), edge),
            2 => self.merge_graphs(found.pop().unwrap(), found.pop().unwrap(), edge),
            _ => eprintln!("Found more than one graph for edge {:?}", edge),
        }
    }

    /// Adds a new graph to the forest with the first edge node as the root.
    fn add_graph(&mut self, edge: CellSet) {
        let graph = Graph::new(self.known, edge);
        self.graphs.insert(graph.root, graph);
    }

    /// Adds the edge to the graph with the root.
    fn add_graph_edge(&mut self, root: Cell, edge: CellSet) {
        self.graphs.get_mut(&root).unwrap().add_edge(edge);
    }

    /// Merges the second graph into the first, connected by the edge.
    fn merge_graphs(&mut self, merge: Cell, into: Cell, edge: CellSet) {
        let other = self.graphs.remove(&merge).unwrap();
        self.graphs.get_mut(&into).unwrap().merge(other, edge);
    }

    fn dump(&self) {
        for graph in self.graphs.values() {
            graph.dump();
        }
    }
}

/// Holds all connected peer cells in a cyclic graph.
struct Graph {
    known: Known,
    // First cell in the graph, used only as a key in the forest.
    root: Cell,
    nodes: CellSet,
    // Edges between each pair of cells.
    edges: HashMap<Cell, CellSet>,
    blues: CellSet,
    greens: CellSet,
}

impl Graph {
    /// Returns a new graph with the first edge node as the root.
    fn new(known: Known, edge: CellSet) -> Self {
        let (root, other) = edge.as_pair().unwrap();
        let mut graph = Graph {
            known,
            root: edge.first().unwrap(),
            nodes: edge,
            edges: HashMap::new(),
            blues: CellSet::empty() + root,
            greens: CellSet::empty() + other,
        };
        *graph.edges.entry(root).or_default() += other;
        *graph.edges.entry(other).or_default() += root;
        graph
    }

    /// Returns true if either edge node is in this graph.
    fn has_either_node(&self, edge: CellSet) -> bool {
        self.nodes.has_any(edge)
    }

    /// Adds the edge to this graph.
    fn add_edge(&mut self, edge: CellSet) {
        if self.nodes.has_all(edge) {
            // this or check for block edges that share a row/column
            return;
        }
        let (first, second) = edge.as_pair().unwrap();
        *self.edges.entry(first).or_default() += second;
        *self.edges.entry(second).or_default() += first;
        if self.nodes.has_all(edge) {
            // created a loop which is fine as long as they have different colors
            if self.blues.has_all(edge) || self.greens.has_all(edge) {
                eprintln!("Graph {} contains an invalid loop {}", self.nodes, edge);
                self.dump();
            }
        } else if self.nodes.has(first) {
            self.nodes += second;
            if self.blues.has(first) {
                self.greens += second;
            } else {
                self.blues += second;
            }
        } else if self.nodes.has(second) {
            self.nodes += first;
            if self.blues.has(second) {
                self.greens += first;
            } else {
                self.blues += first;
            }
        } else {
            eprintln!("Graph {} contains neither edge node {}", self.nodes, edge);
        }
    }

    /// Merges the other graph into this graph.
    ///
    /// The graphs must have no nodes or edges in common.
    fn merge(&mut self, other: Graph, edge: CellSet) {
        debug_assert!(self.nodes.has_none(other.nodes));
        debug_assert!(self.nodes.has_exactly(1, edge));
        debug_assert!(other.nodes.has_exactly(1, edge));

        let (first, second) = edge.as_pair().expect("edge must have two nodes");
        let first_color = self.node_color(first).or(other.node_color(first));
        let second_color = self.node_color(second).or(other.node_color(second));

        if first_color.expect("first edge node must have a color")
            == second_color.expect("second edge node must have a color")
        {
            self.blues |= other.greens;
            self.greens |= other.blues;
        } else {
            self.blues |= other.blues;
            self.greens |= other.greens;
        }

        self.nodes |= other.nodes;
        for (node, edges) in other.edges {
            self.edges.insert(node, edges);
        }
    }

    /// Returns the color of the node if it's in this graph.
    fn node_color(&self, node: Cell) -> Option<Color> {
        if self.blues.has(node) {
            Some(Color::Blue)
        } else if self.greens.has(node) {
            Some(Color::Green)
        } else {
            None
        }
    }

    /// Returns the first color that has at least two nodes that share a house.u
    ///
    /// Could keep a flag for each color and update it each time a node is added
    /// to a color set. It only needs to be set true once so it can short-circuit.
    ///
    /// `self.erase_blues = self.erase_blues || node.sees_any(self.blues)`
    ///
    /// This would need to be checked when merging graphs, too.
    fn has_same_color_nodes_in_one_house(&self) -> Option<Color> {
        for house in House::iter() {
            if self.blues.has_at_least(2, house.cells()) {
                return Some(Color::Blue);
            } else if self.greens.has_at_least(2, house.cells()) {
                return Some(Color::Green);
            }
        }

        None
    }

    /// Returns true if at least ooe node of each color sees the cell.
    fn both_colors_see_cell(&self, cell: Cell) -> bool {
        cell.sees_any(self.blues) && cell.sees_any(self.greens)
    }

    /// Returns the nodes of the given color.
    fn nodes_of_color(&self, color: Color) -> CellSet {
        match color {
            Color::Blue => self.blues,
            Color::Green => self.greens,
        }
    }

    fn dump(&self) {
        println!("Graph {} {}: {}", self.known, self.root, self.nodes);
        println!("  blue : {}", self.blues);
        println!("  green: {}", self.greens);
        for (cell, peers) in &self.edges {
            println!("  {} -> {}", cell, peers);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Color {
    Blue,
    Green,
}

impl Color {
    fn other(&self) -> Self {
        match self {
            Color::Blue => Color::Green,
            Color::Green => Color::Blue,
        }
    }

    fn flip(&mut self) {
        *self = self.other()
    }
}
