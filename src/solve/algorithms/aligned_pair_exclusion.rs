use super::*;

pub fn find_aligned_pair_exclusion(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();
    let als_sets = find_als_sets(board);

    let candidate_cells: Vec<Cell> = Cell::iter()
        .filter(|cell| {
            let count = board.candidates(*cell).len();
            (2..=5).contains(&count)
        })
        .collect();

    for (index, &first) in candidate_cells.iter().enumerate() {
        let first_candidates = board.candidates(first);
        for &second in candidate_cells.iter().skip(index + 1) {
            let second_candidates = board.candidates(second);

            let aligned = first.peers().has(second);
            let mut allowed_first = DigitSet::empty();
            let mut allowed_second = DigitSet::empty();
            let mut any_pair = false;
            let mut invalid_als_first: [Vec<usize>; 9] = std::array::from_fn(|_| Vec::new());
            let mut invalid_als_second: [Vec<usize>; 9] = std::array::from_fn(|_| Vec::new());

            for d1 in first_candidates {
                for d2 in second_candidates {
                    if aligned && d1 == d2 {
                        continue;
                    }
                    if let Some(als_index) = pair_invalid_by_als(first, second, d1, d2, &als_sets) {
                        invalid_als_first[d1.usize()].push(als_index);
                        invalid_als_second[d2.usize()].push(als_index);
                        continue;
                    }
                    any_pair = true;
                    allowed_first += d1;
                    allowed_second += d2;
                }
            }

            if !any_pair {
                continue;
            }

            let remove_first = first_candidates - allowed_first;
            let remove_second = second_candidates - allowed_second;

            if remove_first.is_empty() && remove_second.is_empty() {
                continue;
            }

            if allowed_first.is_empty() || allowed_second.is_empty() {
                continue;
            }

            let mut action = Action::new(Strategy::AlignedPairExclusion);
            if !remove_first.is_empty() {
                action.erase_digits(first, remove_first);
            }
            if !remove_second.is_empty() {
                action.erase_digits(second, remove_second);
            }
            action.clue_cell_for_digits(Verdict::Primary, first, first_candidates);
            action.clue_cell_for_digits(Verdict::Secondary, second, second_candidates);
            add_als_clues(
                &mut action,
                &als_sets,
                &invalid_als_first,
                &invalid_als_second,
                remove_first,
                remove_second,
            );

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

#[derive(Clone)]
struct Als {
    cells: CellSet,
    digits: DigitSet,
    digit_cells: [CellSet; 9],
}

fn find_als_sets(board: &Board) -> Vec<Als> {
    let mut als_sets = Vec::new();

    for house in House::iter() {
        let cells: Vec<Cell> = house
            .cells()
            .iter()
            .filter(|cell| board.candidates(*cell).len() >= 2)
            .collect();
        let candidates: Vec<DigitSet> = cells.iter().map(|cell| board.candidates(*cell)).collect();
        let cell_count = cells.len();
        if cell_count == 0 {
            continue;
        }

        let total_masks = 1usize << cell_count;
        for mask in 1..total_masks {
            let size = mask.count_ones() as usize;
            if size == 0 {
                continue;
            }

            let mut digits = DigitSet::empty();
            for (index, cell_digits) in candidates.iter().enumerate() {
                if (mask & (1usize << index)) != 0 {
                    digits |= *cell_digits;
                }
            }

            if digits.len() != size + 1 {
                continue;
            }

            let mut subset_cells = CellSet::empty();
            let mut digit_cells = [CellSet::empty(); 9];
            for (index, cell_digits) in candidates.iter().enumerate() {
                if (mask & (1usize << index)) != 0 {
                    let cell = cells[index];
                    subset_cells.add(cell);
                    for digit in *cell_digits {
                        digit_cells[digit.usize()].add(cell);
                    }
                }
            }

            als_sets.push(Als {
                cells: subset_cells,
                digits,
                digit_cells,
            });
        }
    }

    als_sets
}

fn pair_invalid_by_als(
    first: Cell,
    second: Cell,
    d1: Digit,
    d2: Digit,
    als_sets: &[Als],
) -> Option<usize> {
    if d1 == d2 {
        return None;
    }

    let first_peers = first.peers();
    let second_peers = second.peers();

    for (index, als) in als_sets.iter().enumerate() {
        if als.cells.has(first) || als.cells.has(second) {
            continue;
        }
        if !als.digits.has(d1) || !als.digits.has(d2) {
            continue;
        }
        if !first_peers.has_all(als.digit_cells[d1.usize()]) {
            continue;
        }
        if !second_peers.has_all(als.digit_cells[d2.usize()]) {
            continue;
        }
        return Some(index);
    }

    None
}

fn add_als_clues(
    action: &mut Action,
    als_sets: &[Als],
    invalid_als_first: &[Vec<usize>; 9],
    invalid_als_second: &[Vec<usize>; 9],
    remove_first: DigitSet,
    remove_second: DigitSet,
) {
    let mut used = vec![false; als_sets.len()];

    for digit in remove_first {
        for &index in &invalid_als_first[digit.usize()] {
            used[index] = true;
        }
    }
    for digit in remove_second {
        for &index in &invalid_als_second[digit.usize()] {
            used[index] = true;
        }
    }

    for (index, als) in als_sets.iter().enumerate() {
        if used[index] {
            action.clue_cells_for_digits(Verdict::Related, als.cells, als.digits);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::io::{Parse, Parser};
    use crate::puzzle::Options;
    use crate::*;

    use super::*;

    #[test]
    fn example_1() {
        let parser = Parse::packed_with_options(Options::all());
        let (board, effects, failed) = parser.parse(
            "097103040030040700000670003273914586986007104154068007700091000009736050360000070",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_aligned_pair_exclusion(&board, false).expect("not found");
        assert_erase!(got, B3, 5);
    }

    #[test]
    fn example_2() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "S9B2b4r2y0r02038i8i5u06022e0509082f2f047v477y070r060502460509080n06022f042f020u017y0705087q060u0706080r7n7r05027z06027n4i070v4713080z2q060304022b099r0v9q024i7n3f6u2v",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_aligned_pair_exclusion(&board, false).expect("not found");
        assert_erase!(got, C2, 1);
    }

    #[test]
    fn example_3() {
        let parser = Parse::packed_with_options(Options::all());
        let (board, effects, failed) = parser.parse(
            "000000370706003000300172009967238500531496728824517936600305000000000403083000000",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_aligned_pair_exclusion(&board, false).expect("not found");
        assert_erase!(got, A1, 4);
    }

    #[test]
    fn example_4() {
        let parser = Parse::packed_with_options(Options::all());
        let (board, effects, failed) = parser.parse(
            "024053600005040002900602005700501030000367500350004001200035009500010003003200850",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_aligned_pair_exclusion(&board, false).expect("not found");
        assert_erase!(got, B1, 8);
    }

    #[test]
    fn example_5() {
        let parser = Parse::packed_with_options(Options::all());
        let (board, effects, failed) = parser.parse(
            "450900000006800900080020030000500000640000075501078000375080149002000607064701023",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_aligned_pair_exclusion(&board, false).expect("not found");
        assert_erase!(got, B1, 1);
    }

    #[test]
    fn example_6() {
        let parser = Parse::packed_with_options(Options::all());
        let (board, effects, failed) = parser.parse(
            "900000106002590000308040090080000070570204069090070040809050603000038900135000000",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_aligned_pair_exclusion(&board, false).expect("not found");
        assert_erase!(got, D1, 6);
    }

    #[test]
    fn eight_cell_aligned_pair() {
        let parser = Parse::wiki().stop_on_error();
        let (board, effects, failed) = parser.parse(
            "S9B1307028308120604821b0906070l1c0814101a1208067o1ca0a0019u109u03ad1g7p08040608017u9o2c88889w9i0o047n05087p069g02012q080309042q0608049u02363m9u01039u0603162i019w9w08",
        );
        assert_eq!(None, failed);
        assert!(!effects.has_errors());

        let got = find_aligned_pair_exclusion(&board, false).expect("not found");
        assert_erase!(got, E5, 2);
    }
}
