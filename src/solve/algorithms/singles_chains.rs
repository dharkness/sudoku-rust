use std::collections::HashMap;

use super::*;

pub fn find_singles_chains(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();
    let ignore = board.cells_with_n_candidates(1);

    for (known, possibles) in Known::iter()
        .map(|known| (known, board.candidate_cells(known) - ignore))
        .filter(|(_, candidates)| !candidates.is_empty())
    {
        let mut nodes = CellSet::empty();
        let mut edges: HashMap<Cell, CellSet> = HashMap::new();

        for cells in House::iter()
            .map(|house| board.house_candidate_cells(house, known))
            .filter(|cells| cells.len() == 2)
        {
            // println!("house {}, cells {}", house, cells);
            nodes |= cells;

            let pair = cells.as_pair().unwrap();
            let (first, second) = pair;
            *edges.entry(first).or_default() += second;
            *edges.entry(second).or_default() += first;
        }

        let candidates = possibles
            & nodes
                .iter()
                .combinations(2)
                .fold(CellSet::empty(), |acc, pair| {
                    acc | (pair[0].peers() & pair[1].peers())
                });

        let mut chains: Vec<Chain> = Vec::new();
        let mut cell_chains: HashMap<Cell, (usize, usize)> = HashMap::new();

        for candidate in candidates {
            let sees = nodes & candidate.peers();

            let mut chain = Chain::new(candidate);
            let mut stack = vec![sees];
            let mut shortest = cell_chains
                .get(&candidate)
                .map_or(usize::MAX, |(_, length)| *length);

            while !stack.is_empty() {
                let pool = stack.last_mut().unwrap();
                if pool.is_empty() || chain.nodes.len() + 1 >= shortest {
                    if !chain.nodes.is_empty() {
                        chain.pop();
                    }
                    stack.pop();
                    continue;
                }

                let node = pool.pop().unwrap();
                if node == candidate || chain.has(node) {
                    continue;
                }

                chain.push(node);
                if sees[node] && chain.is_mismatched() {
                    if chain.all_nodes_in_same_block() {
                        // degenerate hidden pair
                        cell_chains.remove(&candidate);
                        break;
                    }

                    shortest = chain.nodes.len();
                    chains.push(chain.clone());
                    (candidates & chain.sees()).iter().for_each(|cell| {
                        cell_chains.insert(cell, (chains.len() - 1, chain.len()));
                    });

                    chain.pop();
                    continue;
                }

                let next = edges[&node] - chain.nodes - candidate;
                if !next.is_empty() {
                    stack.push(next);
                } else {
                    chain.pop();
                }
            }
        }

        let mut grouped: HashMap<usize, CellSet> = HashMap::new();
        cell_chains.iter().for_each(|(cell, (index, _))| {
            *grouped.entry(*index).or_default() += *cell;
        });

        for (_, cells) in grouped {
            let mut action = Action::new(Strategy::SinglesChain);
            action.erase_cells(cells, known);

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

#[derive(Debug, Clone)]
struct Chain {
    candidate: Cell,
    nodes: CellSet,
    colors: Colors,

    stack: Vec<Cell>,
    end: Option<Cell>,
    color: Color,
}

impl Chain {
    pub fn new(candidate: Cell) -> Self {
        Self {
            candidate,
            nodes: CellSet::empty(),
            colors: Colors::new(),
            stack: Vec::new(),
            end: None,
            color: Color::Green,
        }
    }

    pub fn is_mismatched(&self) -> bool {
        match self.color {
            Color::Red => false,
            Color::Green => true,
        }
    }

    pub fn all_nodes_in_same_block(&self) -> bool {
        let mut block: Option<House> = None;

        for cell in self.nodes {
            match block {
                None => block = Some(cell.block()),
                Some(b) => {
                    if b != cell.block() {
                        return false;
                    }
                }
            }
        }

        true
    }

    pub fn has(&self, node: Cell) -> bool {
        self.nodes.has(node)
    }

    pub fn push(&mut self, node: Cell) {
        self.color.flip();
        self.end = Some(node);

        self.nodes += node;
        self.colors.add(node, self.color);
        self.stack.push(node);
    }

    pub fn pop(&mut self) {
        if let Some(end) = self.end {
            self.stack.pop();
            self.color.flip();
            self.nodes -= end;
            self.colors.remove(end);
            self.end = self.stack.last().copied();
        }
    }

    pub fn len(&self) -> usize {
        self.stack.len() - 1
    }

    pub fn sees(&self) -> CellSet {
        self.stack.first().unwrap().peers() & self.stack.last().unwrap().peers()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Color {
    Red,
    Green,
}

impl Color {
    pub fn flip(&mut self) {
        match self {
            Color::Red => *self = Color::Green,
            Color::Green => *self = Color::Red,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Colors((CellSet, CellSet));

impl Colors {
    pub fn new() -> Self {
        Self((CellSet::empty(), CellSet::empty()))
    }

    pub fn add(&mut self, node: Cell, color: Color) {
        match color {
            Color::Red => self.0 .0 += node,
            Color::Green => self.0 .1 += node,
        }
    }

    pub fn remove(&mut self, cell: Cell) {
        self.0 .0 -= cell;
        self.0 .1 -= cell;
    }
}
