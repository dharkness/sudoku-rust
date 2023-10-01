use super::*;

pub fn find_bugs(board: &Board) -> Option<Effects> {
    let mut effects = Effects::new();

    let pairs = board.cells_with_n_candidates(2);
    let triples = board.cells_with_n_candidates(3);
    if pairs.is_empty() || triples.size() != 1 {
        return None;
    }

    for count in [1, 4, 5, 6, 7, 8, 9] {
        if !board.cells_with_n_candidates(count).is_empty() {
            return None;
        }
    }

    let triple = triples.as_single().unwrap();
    let candidates = board.candidates(triple);
    let mut eliminated = KnownSet::empty();

    for known in candidates {
        for house in triple.houses() {
            if board.house_candidate_cells(house, known).size() == 2 {
                // removing this candidate will not create a BUG
                eliminated += known;
                break;
            }
        }
    }

    if eliminated.size() == 2 {
        let solution = (candidates - eliminated).as_single().unwrap();
        effects.add_set(Strategy::Bug, triple, solution);
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}
