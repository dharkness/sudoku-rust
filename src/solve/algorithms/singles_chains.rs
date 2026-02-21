use std::collections::HashMap;

use super::*;

pub fn find_singles_chains(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();
    let ignore = board.cells_with_n_candidates(1);

    for (digit, possibles) in Digit::iter()
        .map(|digit| (digit, board.candidate_cells(digit) - ignore))
        .filter(|(_, candidates)| !candidates.is_empty())
    {
        let mut nodes = CellSet::empty();
        let mut edges: HashMap<Cell, CellSet> = HashMap::new();

        for cells in House::iter()
            .map(|house| board.house_candidate_cells(house, digit))
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
            & nodes.pair_iter().fold(CellSet::empty(), |acc, (c1, c2)| {
                acc | (c1.peers() & c2.peers())
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

        let mut grouped = vec![CellSet::empty(); chains.len()];
        for (cell, (index, _)) in cell_chains.iter() {
            if let Some(group) = grouped.get_mut(*index) {
                *group += *cell;
            }
        }

        for (index, cells) in grouped.iter().enumerate() {
            if cells.is_empty() {
                continue;
            }
            let chain = &chains[index];
            let mut action = Action::new(Strategy::SinglesChain);
            action.erase_cells(*cells, digit);
            let red = chain.colors.red();
            let green = chain.colors.green();
            if !red.is_empty() {
                action.clue_cells_for_digit(Verdict::Primary, red, digit);
            }
            if !green.is_empty() {
                action.clue_cells_for_digit(Verdict::Secondary, green, digit);
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

    pub fn red(&self) -> CellSet {
        self.0 .0
    }

    pub fn green(&self) -> CellSet {
        self.0 .1
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

#[cfg(test)]
mod tests {
    use crate::io::{Parse, Parser};
    use crate::*;

    use super::*;

    fn assert_chain_clues(action: &Action, digit: Digit) {
        let mut has_primary = false;
        let mut has_secondary = false;
        for (_, clue_digit, verdict) in action.collect_clues() {
            if clue_digit == digit {
                match verdict {
                    Verdict::Primary => has_primary = true,
                    Verdict::Secondary => has_secondary = true,
                    _ => (),
                }
            }
        }
        assert!(has_primary, "expected primary clues for {}", digit);
        assert!(has_secondary, "expected secondary clues for {}", digit);
    }

    #[test]
    fn rule_2_example() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "S9B0l2d04080e2d030i0f7r059f2b2f060h04020o080f0d090o2q2b0z069h0h0e2d047o0n7r7p039f0f089f7w057v0d7p0e030l7n0f0h070e7n7r9f065z9m02be07047r020n478206bm0h0f029e0d05012e7q",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        if let Some(got) = find_singles_chains(&board, true) {
            let mut action = Action::new(Strategy::SinglesChain);
            action.erase_cells(cells![A6 B3], digit!(7));
            action.clue_cells_for_digit(Verdict::Primary, cells![A2 D5], digit!(7));
            action.clue_cells_for_digit(Verdict::Secondary, cells![B5 D2], digit!(7));

            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }

    #[test]
    fn rule_4_example() {
        let parser = Parse::wiki();
        let (board, _effects, _) = parser.parse(
            "S9B029y6e5y0d0acy12060d9y6e066e02cy0a5y5u010f5y092u5w1404032q6a0a0b09060d5u0a040b5y0f2e0e095y5u0f09055y0d5w0o01050h0d0b010f03070i0i0b2e042e080a0f0e062e0a0i2u2u0d0h02",
        );

        if let Some(got) = find_singles_chains(&board, false) {
            let action = got
                .actions()
                .iter()
                .find(|action| action.erases(cell!(B5), digit!(3)))
                .unwrap_or_else(|| panic!("expected erase of 3 from B5"));
            assert_chain_clues(action, digit!(3));
        } else {
            panic!("not found");
        }
    }
}
