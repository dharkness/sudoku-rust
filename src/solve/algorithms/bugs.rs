use super::*;

pub fn find_bugs(board: &Board) -> Option<Effects> {
    let mut effects = Effects::new();

    let pairs = board.cells_with_n_candidates(2);
    let triples = board.cells_with_n_candidates(3);
    if pairs.is_empty() || triples.len() != 1 {
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
            if board.house_candidate_cells(house, known).len() == 2 {
                // removing this candidate will not create a BUG
                eliminated += known;
                break;
            }
        }
    }

    if eliminated.len() == 2 {
        let solution = (candidates - eliminated).as_single().unwrap();
        let mut action = Action::new_set(Strategy::Bug, triple, solution);
        action.clue_cells_for_known(
            Color::Blue,
            triple.peers() & board.candidate_cells(solution),
            solution,
        );
        effects.add_action(action);
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::io::{Parse, Parser};
    use crate::layout::cells::cell::cell;
    use crate::layout::cells::cell_set::cells;
    use crate::layout::values::known::known;

    use super::*;

    #[test]
    fn test() {
        let parser = Parse::wiki().stop_on_error();
        let (board, ..) = parser.parse(
            "418121030511090hg10i110kg109410681210ag10c81210h06411181210341g1050h1109g10o0o2111038105411105410h8109g121030s0o9018032141g1840c4190180hg12103842103g105418111090h",
        );

        let mut action = Action::new_set(Strategy::Bug, cell!("G1"), known!("3"));
        action.clue_cells_for_known(Color::Blue, cells!("C1 G2 G4 H1"), known!("3"));

        let effects = find_bugs(&board).unwrap();
        assert_eq!(action, effects.actions()[0]);
    }
}
