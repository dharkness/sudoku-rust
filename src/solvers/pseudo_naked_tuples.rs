use super::*;

pub fn find_pseudo_naked_tuples(
    board: &Board,
    house: House,
    pseudo: PseudoCell,
    size: usize,
    action: &mut Action,
) {
    let peers = house.cells() - pseudo.cells;
    peers
        .iter()
        .map(|cell| (cell, board.candidates(cell)))
        .filter(|(_, knowns)| (2..=size).contains(&knowns.size()))
        .combinations(size - 1)
        .for_each(|peer_knowns| {
            let tuple = PseudoTuple::new(pseudo, &peer_knowns);
            let known_sets: Vec<KnownSet> = peer_knowns
                .iter()
                .map(|(_, ks)| *ks)
                .chain([pseudo.knowns])
                .collect();
            let knowns = known_sets.iter().copied().union() as KnownSet;
            if knowns.size() != size
                || naked_tuples::is_degenerate(&known_sets, size, 2)
                || naked_tuples::is_degenerate(&known_sets, size, 3)
            {
                return;
            }

            let cells = peers - peer_knowns.iter().map(|(c, _)| *c).union() as CellSet;

            knowns
                .iter()
                .for_each(|k| action.erase_cells(cells & board.candidate_cells(k), k));
        });
}

struct PseudoTuple<'a> {
    pub pseudo: PseudoCell,
    pub cells: CellSet,
    pub knowns: KnownSet,
    pub cell_knowns: &'a Vec<(Cell, KnownSet)>,
}

impl PseudoTuple<'_> {
    pub fn new(pseudo: PseudoCell, cell_knowns: &'_ Vec<(Cell, KnownSet)>) -> PseudoTuple<'_> {
        PseudoTuple {
            pseudo,
            cells: cell_knowns
                .iter()
                .map(|(c, _)| *c)
                .chain([pseudo.pseudo])
                .union() as CellSet,
            knowns: cell_knowns
                .iter()
                .map(|(_, k)| *k)
                .chain([pseudo.knowns])
                .union() as KnownSet,
            cell_knowns,
        }
    }

    pub fn find_degenerates(
        &self,
        size: usize,
        smaller_size: usize,
    ) -> Option<Vec<Vec<(Cell, KnownSet)>>> {
        if size <= smaller_size {
            return None;
        }
        let found = self
            .cell_knowns
            .iter()
            .combinations(smaller_size)
            .filter_map(|subset| {
                let knowns = subset.iter().map(|(_, ks)| *ks).union();
                if knowns.size() > smaller_size {
                    return None;
                }
                if subset.iter().any(|(c, _)| *c == self.pseudo.pseudo) {
                    return None;
                }
                Some(
                    subset
                        .iter()
                        .map(|(c, ks)| (*c, *ks))
                        .collect::<Vec<(Cell, KnownSet)>>(),
                )
            })
            .collect::<Vec<Vec<(Cell, KnownSet)>>>();

        if found.is_empty() {
            None
        } else {
            Some(found)
        }
    }
}

fn find_degenerates(
    cell_knowns: &[(Cell, KnownSet)],
    size: usize,
    smaller_size: usize,
) -> Option<Vec<Vec<&(Cell, KnownSet)>>> {
    if size > smaller_size {
        let mut found: Vec<Vec<&(Cell, KnownSet)>> = vec![];
        cell_knowns
            .iter()
            .combinations(smaller_size)
            .for_each(|subset| {
                let knowns = subset.iter().map(|(_, ks)| *ks).union();
                if knowns.size() <= smaller_size {
                    found.push(subset);
                }
            });
        if !found.is_empty() {
            return Some(found);
        }
    }
    None
}
