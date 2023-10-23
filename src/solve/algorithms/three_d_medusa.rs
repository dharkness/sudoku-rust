use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::{Rc, Weak};

use super::*;

pub fn find_three_d_medusa(board: &Board) -> Option<Effects> {
    let mut effects = Effects::new();
    let mut forest = Forest::new();

    for known in Known::iter() {
        // println!("\nKNOWN {} =================================\n", known);

        for (house, edge) in board.house_candidates_with_n_candidate_cells(2, known) {
            if !(house.is_block() && (edge.share_row() || edge.share_column())) {
                // println!("edge {}", edge);
                forest.add_strong_link(edge, known);
                // println!("size {}", forest.all_graphs.len());
            }
        }
    }

    // println!("\nforest\n");
    // forest.dump();

    // Simple Coloring

    for graph in forest.all_graphs.values() {
        // println!("\nlet's go!\n");
        let the_graph = graph.borrow();
        // the_graph.dump();
        let coloring = the_graph.coloring();
        // coloring.dump();
        let possibles = board.candidate_cells(the_graph.root.known);

        // rule 2 - two of one color in a house
        if let Some(color) = coloring.has_same_color_nodes_in_one_house() {
            let known = the_graph.root.known;
            let mut action = Action::new(Strategy::SimpleColoring);

            action.erase_cells(coloring.cells_of_color(color), known);
            action.set_cells(coloring.cells_of_color(color.other()), known);
            effects.add_action(action);
        }

        // rule 4 - candidate sees two nodes of opposite colors
        let erase = possibles
            .iter()
            .filter(|c| coloring.both_colors_see_cell(*c))
            .union() as CellSet;

        // Multi-Coloring
        // TODO find graphs that contain cells to erase; those graphs can be solved as above
        effects.add_erase_cells(Strategy::SimpleColoring, erase, the_graph.root.known);
    }

    // 3D Medusa

    for (cell, pair) in board.cell_candidates_with_n_candidates(2) {
        // println!("\nbi-value {} {}", cell, pair);
        forest.add_bi_value(cell, pair);
    }

    for graph in forest.all_graphs.values() {
        // println!("\nlet's go!\n");
        let the_graph = graph.borrow();
        // the_graph.dump();
        let coloring = the_graph.coloring();
        // coloring.dump();
        // let possibles = board.candidate_cells(the_graph.known);

        // if the_graph.root.known == Known::from("3") && the_graph.root.cell == Cell::from("H2") {
        //     println!("ready?");
        //     the_graph.dump();
        //     coloring.dump();
        //     println!();
        // }

        let known_cell_colors = coloring.cell_colors_by_known();
        if known_cell_colors.len() < 2 {
            // already handled above by simple coloring
            continue;
        }

        let mut erase_graph_color: Option<Color> = None;
        let mut type_three = Action::new(Strategy::ThreeDMedusa);

        let cell_key_colors = coloring.key_colors_by_cell();
        for (cell, key_colors) in cell_key_colors {
            let color_keys = key_colors.iter().fold(
                HashMap::new(),
                |mut acc: HashMap<Color, KnownSet>, KeyColor { key, color }| {
                    acc.entry(*color).or_default().add(key.known);
                    acc
                },
            );

            // stop checking once the graph is solved
            if erase_graph_color.is_none() {
                // rule 1 - two of one color in a cell
                for (color, knowns) in &color_keys {
                    if knowns.len() >= 2 {
                        erase_graph_color = Some(*color);
                        break;
                    }
                }
            }
        }

        // rule 3 - both colors in a cell
        // remove all other candidates in that cell
        known_cell_colors.iter().combinations(2).for_each(|pair| {
            let (k1, (blue1, green1)) = pair[0];
            let (k2, (blue2, green2)) = pair[1];

            for cell in (*blue1 & *green2) | (*blue2 & *green1) {
                type_three.erase_knowns(cell, board.candidates(cell) - *k1 - *k2);
            }
        });

        if erase_graph_color.is_none() {
            // rule 2 - two of one color in a house
        }
        if let Some(erase_color) = erase_graph_color {
            let mut action = Action::new(Strategy::ThreeDMedusa);

            for Key { cell, known } in match erase_color {
                Color::Blue => &coloring.blues,
                Color::Green => &coloring.greens,
            } {
                action.erase(*cell, *known);
            }
            for Key { cell, known } in match erase_color.other() {
                Color::Blue => &coloring.blues,
                Color::Green => &coloring.greens,
            } {
                action.set(*cell, *known);
            }
            effects.add_action(action);
        }
        effects.add_action(type_three);

        // let (blues, greens, both) = coloring.cells_by_colors();
        // for cell in both {
        //     action.erase_knowns(cell, possibles - the_graph.known);
        // }

        // let cell_colors = coloring.cell_colors();
        // let cells = coloring.cells_with_both_colors();
        // if !cells.is_empty() {}
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

struct Forest {
    /// The set of all unique graphs.
    ///
    /// Map by root node key for easier merging?
    all_graphs: HashMap<Key, Rc<RefCell<Graph>>>,

    /// Links every graph to all of its node keys for lookup.
    key_graphs: HashMap<Key, Rc<RefCell<Graph>>>,
}

impl Forest {
    fn new() -> Self {
        Self {
            all_graphs: HashMap::new(),
            key_graphs: HashMap::new(),
        }
    }

    /// Adds an edge to an existing or new graph, joining two graphs if necessary.
    ///
    /// This edge represents a strong link between two cells,
    /// meaning that exactly one of them must solve to the known.
    fn add_strong_link(&mut self, edge: CellSet, known: Known) {
        // let seven = Known::from("7");
        // if known == seven && (edge.has(Cell::from("C4")) || edge.has(Cell::from("B5"))) {
        //     self.dump();
        //     println!("add_strong_link {} {}", edge, known);
        //     println!(
        //         "{:?}",
        //         self.key_graphs
        //             .iter()
        //             .filter(|(k, _)| k.known == seven)
        //             .map(|(k, _)| format!("{}", k))
        //             .collect_vec()
        //     );
        //     println!();
        // }
        match self.pick_graph_action(Key::pair_from_edge(edge, known)) {
            GraphAction::Create(root, other) => {
                let graph = Rc::new(RefCell::new(Graph::new(root, other)));
                // graph.borrow().dump();
                self.all_graphs.insert(root, graph.clone());
                self.key_graphs.insert(root, graph.clone());
                self.key_graphs.insert(other, graph);
            }
            GraphAction::Extend(graph, key_from, key_to) => {
                // self.dump();
                graph.borrow_mut().add_edge(key_from, key_to);
                // graph.borrow().dump();
                self.key_graphs.insert(key_to, graph);
            }
            GraphAction::Loop(graph, key_from, key_to) => {
                // self.dump();
                graph.borrow_mut().add_loop(key_from, key_to);
                // graph.borrow().dump();
                self.key_graphs.insert(key_to, graph);
            }
            GraphAction::Merge(graph_from, key_from, key_to, graph_to) => {
                self.merge(graph_from, key_from, key_to, graph_to);
            }
        }
    }

    fn add_bi_value(&mut self, cell: Cell, pair: KnownSet) {
        if let GraphAction::Merge(graph_from, key_from, key_to, graph_to) =
            self.pick_graph_action(Key::pair_from_bi_value(cell, pair))
        {
            self.merge(graph_from, key_from, key_to, graph_to);
        }
    }

    fn merge(
        &mut self,
        graph_from: Rc<RefCell<Graph>>,
        key_from: Key,
        key_to: Key,
        graph_to: Rc<RefCell<Graph>>,
    ) {
        self.all_graphs.remove(&graph_to.borrow().root);
        // self.key_graphs.remove(&key_to);
        for key in graph_to.borrow().nodes.keys() {
            self.key_graphs.insert(*key, graph_from.clone());
        }
        graph_from.borrow_mut().merge(key_from, key_to, graph_to);
        // graph_from.borrow().dump();
    }

    fn pick_graph_action(&self, (key_one, key_two): (Key, Key)) -> GraphAction {
        let graph_one = self.key_graphs.get(&key_one);
        let graph_two = self.key_graphs.get(&key_two);

        match (graph_one, graph_two) {
            // the key order is arbitrary
            (None, None) => GraphAction::Create(key_one, key_two),

            // extend key one's graph to key two
            (Some(graph_one), None) => GraphAction::Extend(graph_one.clone(), key_one, key_two),

            // extend key two's graph to key one
            (None, Some(graph_two)) => GraphAction::Extend(graph_two.clone(), key_two, key_one),

            // close a loop in one graph or merge the smaller graph into the larger one
            (Some(graph_one), Some(graph_two)) => {
                if graph_one.borrow().root == graph_two.borrow().root {
                    GraphAction::Loop(graph_one.clone(), key_one, key_two)
                } else if graph_one.borrow().len() >= graph_two.borrow().len() {
                    GraphAction::Merge(graph_one.clone(), key_one, key_two, graph_two.clone())
                } else {
                    GraphAction::Merge(graph_two.clone(), key_two, key_one, graph_one.clone())
                }
            }
        }
    }

    fn dump(&self) {
        for graph in self.all_graphs.values() {
            graph.borrow().dump();
        }
    }
}

enum GraphAction {
    /// Create a new graph with a single edge.
    Create(Key, Key),
    /// Extend an existing graph to include a new edge.
    Extend(Rc<RefCell<Graph>>, Key, Key),
    /// Close a loop on an existing graph.
    Loop(Rc<RefCell<Graph>>, Key, Key),
    /// Merge two graphs connected by an edge.
    Merge(Rc<RefCell<Graph>>, Key, Key, Rc<RefCell<Graph>>),
}

struct Graph {
    /// First node belonging to the graph for identification.
    root: Key,
    cells: CellSet,
    knowns: KnownSet,
    /// Every node belonging to the graph.
    nodes: HashMap<Key, Rc<RefCell<Node>>>,
}

impl Graph {
    fn new(root: Key, other: Key) -> Self {
        let mut graph = Self {
            root,
            cells: CellSet::empty() + root.cell + other.cell,
            knowns: KnownSet::empty() + root.known,
            nodes: HashMap::new(),
        };
        // let mut node_one = Rc::new(Node::new(key_one));
        // let mut node_two = Rc::new(Node::new(key_two));
        // graph.nodes.insert(key_one, node_one.clone());
        // graph.nodes.insert(key_two, node_two.clone());
        let mut root_node = graph.add_node(root);
        let mut other_node = graph.add_node(other);
        graph.link_nodes(&mut root_node, &mut other_node);
        graph
    }

    fn len(&self) -> usize {
        self.nodes.len()
    }

    fn node(&self, key: Key) -> Option<Rc<RefCell<Node>>> {
        self.nodes.get(&key).cloned()
    }

    // get_node, creating if necessary?
    fn add_node(&mut self, key: Key) -> Rc<RefCell<Node>> {
        let node = Rc::new(RefCell::new(Node::new(key)));
        self.cells += key.cell;
        self.nodes.insert(key, node.clone());
        node.clone()
    }

    fn add_edge(&mut self, key_from: Key, key_to: Key) {
        if let Some(node_from) = self.nodes.get(&key_from) {
            let node_to = Rc::new(RefCell::new(Node::new(key_to)));
            node_from.borrow_mut().link(node_to.clone());
            node_to.borrow_mut().link(node_from.clone());
            self.cells += key_to.cell;
            self.nodes.insert(key_to, node_to);
        } else {
            panic!("graph does not contain from node {}", key_from);
        }
    }

    fn add_loop(&mut self, key_from: Key, key_to: Key) {
        if let Some(node_from) = self.nodes.get(&key_from) {
            if let Some(node_to) = self.nodes.get(&key_to) {
                node_from.borrow_mut().link(node_to.clone());
                node_to.borrow_mut().link(node_from.clone());
            } else {
                panic!("graph does not contain to node {}", key_from);
            }
        } else {
            panic!("graph does not contain from node {}", key_from);
        }
    }

    fn link_nodes(&mut self, node_from: &mut Rc<RefCell<Node>>, node_to: &mut Rc<RefCell<Node>>) {
        node_from.borrow_mut().link(node_to.clone());
        node_to.borrow_mut().link(node_from.clone());
    }

    fn merge(&mut self, key_from: Key, key_to: Key, other: Rc<RefCell<Graph>>) {
        if let Some(node_from) = self.node(key_from) {
            if let Some(node_to) = other.borrow().node(key_to) {
                node_from.borrow_mut().link(node_to.clone());
                node_to.borrow_mut().link(node_from.clone());

                self.cells |= other.borrow().cells;
                self.knowns |= other.borrow().knowns;
                for (key, node) in &other.borrow().nodes {
                    self.nodes.insert(*key, node.clone());
                }
            } else {
                panic!("from graph does not contain from node {}", key_from);
            }
        } else {
            panic!("to graph does not contain to node {}", key_from);
        }
    }

    fn coloring(&self) -> Coloring {
        let mut coloring = Coloring::new();
        self.nodes
            .get(&self.root)
            .unwrap()
            .borrow()
            .add_coloring(Color::Blue, &mut coloring);

        coloring
    }

    fn dump(&self) {
        println!(
            "Graph {} - {} {}\n  {:?}",
            self.root,
            self.knowns,
            self.cells,
            self.nodes.keys().map(|k| format!("{}", k)).collect_vec()
        );
        for (key, node) in &self.nodes {
            println!("  {}", node.borrow());
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
struct Key {
    cell: Cell,
    known: Known,
}

impl Key {
    fn new(cell: Cell, known: Known) -> Self {
        Self { cell, known }
    }

    fn pair_from_edge(edge: CellSet, known: Known) -> (Self, Self) {
        let (cell_one, cell_two) = edge.as_pair().expect("edge must have two cells");
        (Self::new(cell_one, known), Self::new(cell_two, known))
    }

    fn pair_from_bi_value(cell: Cell, candidates: KnownSet) -> (Self, Self) {
        let (known_one, known_two) = candidates
            .as_pair()
            .expect("bi-value cell must have two candidates");
        (Self::new(cell, known_one), Self::new(cell, known_two))
    }
}

impl PartialOrd<Self> for Key {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.cell.partial_cmp(&other.cell) {
            Some(Ordering::Equal) => self.known.partial_cmp(&other.known),
            result => result,
        }
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.cell, self.known)
    }
}

#[derive(Debug)]
struct Node {
    key: Key,
    edges: Vec<Weak<RefCell<Node>>>,
}

impl Node {
    fn new(key: Key) -> Self {
        Self {
            key,
            edges: Vec::new(),
        }
    }

    fn link(&mut self, other: Rc<RefCell<Node>>) {
        self.edges.push(Rc::downgrade(&other));
    }

    fn add_coloring(&self, color: Color, coloring: &mut Coloring) {
        coloring.add(self.key, color);
        let other_color = color.other();
        for node in &self.edges {
            let upgraded = node.upgrade().expect("node was deallocated");
            let borrowed = upgraded.borrow();
            if coloring.add_edge(borrowed.key, other_color) {
                // println!(
                //     "adding {:?} {} -> {:?} {}",
                //     color, self.key, other_color, borrowed.key
                // );
                // coloring.dump();
                borrowed.add_coloring(other_color, coloring);
            } else {
                // println!(
                //     "skipping {:?} {} -> {:?} {}",
                //     coloring.key_color(self.key),
                //     self.key,
                //     coloring.key_color(borrowed.key),
                //     borrowed.key
                // );
                // coloring.dump();
            }
        }
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for Node {}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

impl PartialOrd<Self> for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ->", self.key);
        for edge in &self.edges {
            write!(
                f,
                " {}",
                edge.upgrade().expect("node was deallocated").borrow().key
            )?;
        }
        Ok(())
    }
}

struct Coloring {
    blues: HashSet<Key>,
    blue_cells: CellSet,
    greens: HashSet<Key>,
    green_cells: CellSet,
}

impl Coloring {
    fn new() -> Self {
        Self {
            blues: HashSet::new(),
            blue_cells: CellSet::empty(),
            greens: HashSet::new(),
            green_cells: CellSet::empty(),
        }
    }

    /// Returns true if the edge was added.
    ///
    /// Panics if already set to a different color.
    fn add_edge(&mut self, key: Key, color: Color) -> bool {
        if let Some(color_to) = self.key_color(key) {
            if color_to != color {
                panic!("edge connects two nodes of the same color");
            }
            false
        } else {
            self.add(key, color);
            true
        }
    }

    /// Returns the color for the key.
    fn key_color(&self, key: Key) -> Option<Color> {
        if self.blues.contains(&key) {
            Some(Color::Blue)
        } else if self.greens.contains(&key) {
            Some(Color::Green)
        } else {
            None
        }
    }

    /// Adds the key to the color set.
    fn add(&mut self, key: Key, color: Color) {
        match color {
            Color::Blue => self.add_blue(key),
            Color::Green => self.add_green(key),
        }
    }

    fn add_blue(&mut self, key: Key) {
        self.blues.insert(key);
        self.blue_cells += key.cell;
    }

    fn add_green(&mut self, key: Key) {
        self.greens.insert(key);
        self.green_cells += key.cell;
    }

    /// Returns the cells with the given color.
    fn cells_of_color(&self, color: Color) -> CellSet {
        match color {
            Color::Blue => self.blue_cells,
            Color::Green => self.green_cells,
        }
    }

    /// Returns the color(s) of each cell.
    fn cell_colors(&self) -> HashMap<Cell, Colors> {
        let mut colors = HashMap::new();
        for Key { cell, known } in self.blues.iter() {
            colors.insert(*cell, Colors::Blue);
        }
        for Key { cell, known } in self.greens.iter() {
            if colors.contains_key(cell) {
                colors.insert(*cell, Colors::Both);
            } else {
                colors.insert(*cell, Colors::Green);
            }
        }
        colors
    }

    /// Returns the color(s) of each cell mapped by each known.
    fn cell_colors_by_known(&self) -> HashMap<Known, (CellSet, CellSet)> {
        let mut colors: HashMap<Known, (CellSet, CellSet)> = HashMap::new();
        for Key { cell, known } in self.blues.iter() {
            colors
                .entry(*known)
                .or_insert((CellSet::empty(), CellSet::empty()))
                .0 += *cell;
        }
        for Key { cell, known } in self.greens.iter() {
            colors
                .entry(*known)
                .or_insert((CellSet::empty(), CellSet::empty()))
                .1 += *cell;
        }
        colors
    }

    /// Returns the keys with their colors grouped by their cells.
    fn key_colors_by_cell(&self) -> HashMap<Cell, HashSet<KeyColor>> {
        let mut keys: HashMap<Cell, HashSet<KeyColor>> = HashMap::new();
        for key in self.blues.iter() {
            keys.entry(key.cell).or_default().insert(KeyColor {
                key: *key,
                color: Color::Blue,
            });
        }
        for key in self.greens.iter() {
            keys.entry(key.cell).or_default().insert(KeyColor {
                key: *key,
                color: Color::Green,
            });
        }
        keys
    }

    /// Returns the cells grouped by their color(s).
    ///
    /// The first set have only blue, the second have only green, and the third have both.
    fn cells_by_colors(&self) -> (CellSet, CellSet, CellSet) {
        let both = self.blue_cells & self.green_cells;
        (self.blue_cells - both, self.green_cells - both, both)
    }

    /// Returns the cells that have both colors.
    fn cells_with_both_colors(&self) -> CellSet {
        self.blue_cells & self.green_cells
    }

    // only works for simple coloring (single known)
    fn has_same_color_nodes_in_one_house(&self) -> Option<Color> {
        for house in House::iter() {
            if self.blue_cells.has_at_least(2, house.cells()) {
                return Some(Color::Blue);
            } else if self.green_cells.has_at_least(2, house.cells()) {
                return Some(Color::Green);
            }
        }

        None
    }

    /// Returns true if at least ooe node of each color sees the cell.
    fn both_colors_see_cell(&self, cell: Cell) -> bool {
        cell.sees_any(self.blue_cells) && cell.sees_any(self.green_cells)
    }

    fn dump(&self) {
        println!(
            "Coloring {}\n  blue : {:?}\n  green: {:?}\n",
            self.blues.len() + self.greens.len(),
            self.blues.iter().map(|k| format!("{}", k)).collect_vec(),
            self.greens.iter().map(|k| format!("{}", k)).collect_vec()
        );
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

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
struct KeyColor {
    key: Key,
    color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Colors {
    Blue,
    Green,
    Both,
}

impl Colors {
    fn is_both(&self) -> bool {
        match self {
            Colors::Both => true,
            _ => false,
        }
    }
}
