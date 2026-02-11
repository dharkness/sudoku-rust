use super::*;

pub fn find_bugs(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    let pairs = board.cells_with_n_candidates(2);
    let triples = board.cells_with_n_candidates(3);
    if pairs.is_empty() || triples.len() != 1 {
        return None;
    }

    // TODO if !(board.unknowns() - pairs - triples).is_empty() { return None; }
    // all calls to cells_with_n_candidates(n) pass 0, 1, 2, or 3, so we could track only n = 0..3 to save space
    for count in [1, 4, 5, 6, 7, 8, 9] {
        if !board.cells_with_n_candidates(count).is_empty() {
            return None;
        }
    }

    let triple = triples.as_single().unwrap();
    let candidates = board.candidates(triple);
    let mut eliminated = DigitSet::empty();

    for digit in candidates {
        for house in triple.houses() {
            if board.house_candidate_cells(house, digit).len() == 2 {
                // removing this candidate will not create a BUG
                eliminated += digit;
                break;
            }
        }
    }

    if eliminated.len() == 2 {
        let solution = (candidates - eliminated).as_single().unwrap();
        let mut action = Action::new_set(Strategy::Bug, triple, solution);
        action.clue_cells_for_digit(
            Verdict::Secondary,
            triple.peers() & board.candidate_cells(solution),
            solution,
        );

        if effects.add_action(action) && single {
            return Some(effects);
        }
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
    use crate::*;

    use super::*;

    #[test]
    fn test() {
        let parser = Parse::wiki().stop_on_error();
        let (board, ..) = parser.parse(
            "418121030511090hg10i110kg109410681210ag10c81210h06411181210341g1050h1109g10o0o2111038105411105410h8109g121030s0o9018032141g1840c4190180hg12103842103g105418111090h",
        );

        if let Some(got) = find_bugs(&board, true) {
            let mut action = Action::new_set(Strategy::Bug, cell!(G1), digit!(3));
            action.clue_cells_for_digit(Verdict::Secondary, cells![C1 G2 G4 H1], digit!(3));

            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }
}
