use super::*;

// Type 1
// .5....... .6.5.42.. ..8.71... 4....36.8 ......... 89.1..7.. 3........ ...2.7.1. .72.3..9.
//
// Type 2
// .85....6. .....47.. .3....1.. .......5. 6...43... .7.82.3.. ...45967. ......... 9.4167..3
//
// http://sudopedia.enjoysudoku.com/Avoidable_Rectangle.html
pub fn find_avoidable_rectangles(board: &Board) -> Option<Effects> {
    let mut effects = Effects::new();

    let candidates = board.solved();

    // type 1
    Rectangle::iter()
        .map(|r| (r, r.cells - candidates))
        .filter_map(|(r, cs)| cs.as_single().map(|c| (r.with_origin(c), c)))
        .filter(|(r, _)| board.value(r.top_right) == board.value(r.bottom_left))
        .filter_map(|(r, c)| board.value(r.bottom_right).known().map(|k| (c, k)))
        .filter(|(c, k)| board.candidates(*c).has(*k))
        .for_each(|(c, k)| effects.add_erase(Strategy::AvoidableRectangle, c, k));

    for rect in Rectangle::iter() {
        if rect.cells.has_any(board.givens()) {
            continue;
        }

        let unsolved = rect.cells - board.knowns();
        if let Some((c1, c2)) = unsolved.as_pair() {
            let houses = c1.common_houses(c2);
            if houses.is_empty() {
                continue;
            }

            if let Some((c3, c4)) = (rect.cells - unsolved).as_pair() {
                let ks1 = board.candidates(c1);
                let ks2 = board.candidates(c2);
                let k3 = board.value(c3).known().unwrap();
                let k4 = board.value(c4).known().unwrap();
                if !(ks1.has(k4) && ks2.has(k3)) {
                    continue;
                }
            } else {
                continue;
            }

            let mut pseudo = board.pseudo_cell(unsolved);
            let solved = board.all_knowns(rect.cells - unsolved);
            pseudo.knowns -= solved;

            if let Some(k) = pseudo.knowns.as_single() {
                // type 2 - naked single
                let mut action = Action::new(Strategy::AvoidableRectangle);

                for house in houses {
                    action.erase_cells(board.house_candidate_cells(house, k) - unsolved, k);
                }

                effects.add_action(action);
            } else {
                // type 3 - naked tuple
                let mut action = Action::new(Strategy::AvoidableRectangle);

                for house in houses {
                    let peers = house.cells() - rect.cells;
                    for size in 2..=4 {
                        peers
                            .iter()
                            .map(|cell| (cell, board.candidates(cell)))
                            .filter(|(_, knowns)| !knowns.has_any(solved))
                            .filter(|(_, knowns)| (2..=size).contains(&knowns.size()))
                            .combinations(size - 1)
                            .for_each(|peer_knowns| {
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

                                let cells =
                                    peers - peer_knowns.iter().map(|(c, _)| *c).union() as CellSet;

                                knowns.iter().for_each(|k| {
                                    action.erase_cells(cells & board.candidate_cells(k), k)
                                });
                            });
                    }
                }

                effects.add_action(action);

                // degenerates should create actions
                // normally, when looking for a naked triple, finding two cells
                // that collectively can only be two of the knowns
                // would be found by looking for naked pairs,
                // but since a pseudo cell is involved, it wouldn't be found.
                // thus, this should report them, maybe combining it with the triple
                // by removing the pair from the pseudo cell as well.
            }
        }
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}
