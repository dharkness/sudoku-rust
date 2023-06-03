use super::*;

pub fn x_wings(board: &Board) -> Option<Effects> {
    let mut effects = Effects::new();

    Known::ALL.into_iter().for_each(|k| {
        check_houses(board, k, House::all_rows(), &mut effects);
        check_houses(board, k, House::all_columns(), &mut effects);
    });

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

fn check_houses(board: &Board, k: Known, houses: &[House], effects: &mut Effects) {
    let candidate_cells = board.candidate_cells(k);
    let candidates = houses
        .iter()
        .map(|house| (*house, house.cells() & candidate_cells))
        .filter(|(_, cells)| cells.size() == 2)
        .map(|(house, cells)| Candidate::new(house, cells))
        .collect::<Vec<_>>();

    distinct_pairs(&candidates).iter().for_each(|(c1, c2)| {
        let crosses = c1.crosses & c2.crosses;
        if crosses.size() != 2 {
            return;
        }

        let main_cells = c1.cells | c2.cells;
        let cross_cells = crosses.cells() & candidate_cells;
        let erase = cross_cells - main_cells;
        if erase.is_empty() {
            return;
        }

        let mut action = Action::new(Strategy::XWing);
        action.erase_cells(erase, k);
        effects.add_action(action);
    });
}

#[derive(Clone, Copy)]
struct Candidate {
    cells: CellSet,
    crosses: HouseSet,
}

impl Candidate {
    pub fn new(house: House, cells: CellSet) -> Self {
        Self {
            cells,
            crosses: house.crossing_houses(cells),
        }
    }
}
