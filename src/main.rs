mod board;
mod cell_set;
mod cell_set2;
mod coord_set;
mod known_set;
mod layout;
mod printers;

use crate::cell_set::cell_from_label;
use crate::layout::generate_code_for_neighbors;
use crate::printers::{print_candidates, print_values};
use coord_set::CoordSet;

fn count_set_bits(mut bits: i16) -> i16 {
    let mut count = 0;
    while bits != 0 {
        count += bits % 2;
        bits >>= 1;
    }
    count
}

const NIBBLE_COUNTS: [i16; 16] = [0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4];

pub fn count_set_bits_by_nibbles(mut bits: i16) -> i16 {
    let mut count = 0;
    while bits != 0 {
        count += NIBBLE_COUNTS[(bits & 0b1111) as usize];
        bits >>= 4;
    }
    count
}

fn main() {
    // generate_code_for_neighbors();
    use_board();
    // use_known_set();
    // use_coord_set();
    // test_perf();
}

fn use_board() {
    let mut board = board::Board::new();

    assert!(!board.is_solved());
    assert_eq!(board.given_count(), 0);
    assert_eq!(board.known_count(), 0);
    assert!(!board.is_given(12));
    assert!(!board.is_known(12));
    assert_eq!(board.value(12), known_set::UNKNOWN);

    board.set_known(cell_from_label("A6"), 5);
    assert!(board.is_known(cell_from_label("A6")));
    assert_eq!(board.value(cell_from_label("A6")), 5);

    let mut board = board::Board::new();
    board.set_given(cell_from_label("A2"), 3);
    board.set_given(cell_from_label("A4"), 6);
    board.set_given(cell_from_label("A9"), 7);
    board.set_given(cell_from_label("B3"), 8);
    board.set_given(cell_from_label("B4"), 7);
    board.set_given(cell_from_label("B7"), 1);
    board.set_given(cell_from_label("C5"), 8);
    board.set_given(cell_from_label("D6"), 3);
    board.set_given(cell_from_label("D7"), 9);
    board.set_given(cell_from_label("E1"), 8);
    board.set_given(cell_from_label("E7"), 3);
    board.set_given(cell_from_label("E8"), 7);
    board.set_given(cell_from_label("F2"), 2);
    board.set_given(cell_from_label("F4"), 4);
    board.set_given(cell_from_label("F9"), 1);
    board.set_given(cell_from_label("G3"), 2);
    board.set_given(cell_from_label("G4"), 9);
    board.set_given(cell_from_label("G9"), 4);
    board.set_given(cell_from_label("H1"), 7);
    board.set_given(cell_from_label("H2"), 5);
    board.set_given(cell_from_label("H3"), 6);
    board.set_given(cell_from_label("H4"), 3);
    board.set_given(cell_from_label("H9"), 9);
    board.set_given(cell_from_label("J3"), 4);
    board.set_given(cell_from_label("J8"), 1);

    board.remove_candidate(cell_from_label("B8"), 3);
    board.remove_candidate(cell_from_label("B9"), 3);
    board.set_known(cell_from_label("B5"), 3);

    print_values(&board);
    print_candidates(&board);
}

fn use_known_set() {
    println!("{}", known_set::to_string(known_set::empty()));
    println!("{}", known_set::to_string(known_set::full()));
    println!("{}", known_set::to_string(known_set::of(&[2, 3, 8])));

    let mut set1 = known_set::of(&[2, 3, 8]);
    let mut set2 = known_set::of(&[1, 3, 4, 7]);

    known_set::add(&mut set1, 5);
    known_set::remove(&mut set2, 4);

    println!("{}", known_set::to_string(known_set::with(set1, 6)));
    println!("{}", known_set::to_string(known_set::without(set2, 3)));

    println!("{}", known_set::debug(known_set::union(set1, set2)));
    println!("{}", known_set::debug(known_set::intersect(set1, set2)));
    println!("{}", known_set::debug(known_set::diff(set1, set2)));
}

fn use_coord_set() {
    println!("{} = {}", 0b111111111, count_set_bits(0b111111111));
    println!("{} = {}", 0b101, count_set_bits(0b101));

    let mut coords = CoordSet::empty();
    coords.add(0);
    coords.add(2);
    coords.add(2);
    coords.add(4);
    println!("has 2: {}", coords.has(2));
    println!("has 3: {}", coords.has(3));
    coords.debug();
    println!("{}", coords);
    println!("{:?}", coords);

    let mut coords2 = coords.clone();
    coords2.remove(2);
    coords2.add(3);
    coords2.add(7);
    coords2.debug();
    println!("{}", coords2);

    let mut coords3 = coords.union(&coords2);
    coords3.add(8);
    coords3.debug();
    println!("{}", coords3);
}

fn test_perf() {
    use rand::Rng;
    use std::time::Instant;

    const SIZE: usize = 100_000_000;

    let now = Instant::now();

    let mut bits: Vec<i16> = vec![0; SIZE];
    let mut sizes: Vec<i16> = vec![0; SIZE];
    let mut rng = rand::thread_rng();

    for slot in bits.iter_mut().take(SIZE) {
        *slot = rng.gen_range(0..1 << 9);
    }

    let elapsed = now.elapsed();
    println!("Allocate: {:.2?}", elapsed);

    let now = Instant::now();
    for i in 0..SIZE {
        sizes[i] = count_set_bits(bits[i]);
    }
    let elapsed = now.elapsed();
    println!("Regular:  {:.2?}", elapsed);

    let now = Instant::now();
    for i in 0..SIZE {
        sizes[i] = count_set_bits_by_nibbles(bits[i]);
    }
    let elapsed = now.elapsed();
    println!("Nibbles:  {:.2?}", elapsed);

    let now = Instant::now();
    for i in 0..SIZE {
        sizes[i] = coord_set::coord_set_size(bits[i]);
    }
    let elapsed = now.elapsed();
    println!("Table:    {:.2?}", elapsed);

    let now = Instant::now();
    let mut xor: i16 = 0;
    for size in sizes.iter().take(SIZE) {
        xor ^= size;
    }
    let elapsed = now.elapsed();
    println!("Xor:      {:09b}", xor);
    println!("Calc:     {:.2?}", elapsed);
}
