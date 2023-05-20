pub type Coord = usize;
pub type Row = Coord;
pub type Column = Coord;
pub type Block = Coord;

pub type Cell = u8;
pub type CellSet = u128;

type Bits = u128;
type Size = u8;

pub const ALL_COORDS: std::ops::Range<Coord> = 0..9;
pub const ALL_CELLS: std::ops::Range<Cell> = 0..CELL_COUNT;

const CELL_COUNT: Size = 81;
const ALL_BITS_SET: Bits = (1 << CELL_COUNT) - 1;
const FULL: CellSet = pack(ALL_BITS_SET, CELL_COUNT);

const BITS_MASK: Bits = (1 << CELL_COUNT) - 1;
const SIZE_SHIFT: Size = 128 - 8;
const MARKER: Bits = ((1 as Bits) << (128 - 10)) + ((1 as Bits) << 82);

pub const fn assert_cell(cell: Cell) {
    assert!(cell < CELL_COUNT);
}

const fn pack(bits: Bits, size: Size) -> CellSet {
    assert!(bits <= BITS_MASK);
    assert!(size <= CELL_COUNT);
    ((size as Bits) << SIZE_SHIFT) + bits
}

pub const fn empty() -> CellSet {
    0
}

pub const fn full() -> CellSet {
    pack(ALL_BITS_SET, CELL_COUNT)
}

pub const fn new(bits: Bits) -> CellSet {
    pack(bits, count_bits(bits))
}

pub fn of_cells(cells: &[Cell]) -> CellSet {
    let mut set = empty();
    for cell in cells {
        add(&mut set, *cell);
    }
    set
}

pub fn of(labels: &[&str]) -> CellSet {
    let mut set = empty();
    for label in labels {
        add(&mut set, cell_from_label(label));
    }
    set
}

pub const fn is_empty(set: CellSet) -> bool {
    set == 0
}

pub const fn size(set: CellSet) -> Size {
    (set >> SIZE_SHIFT) as Size
}

pub const fn bits(set: CellSet) -> Bits {
    set & BITS_MASK
}

pub const fn has(set: CellSet, cell: Cell) -> bool {
    assert_cell(cell);
    set & (1 << cell) != 0
}

pub fn add(set: &mut CellSet, cell: Cell) {
    assert_cell(cell);
    if !has(*set, cell) {
        *set |= 1 << cell;
        *set += 1 << SIZE_SHIFT;
    }
}

pub const fn with(set: CellSet, cell: Cell) -> CellSet {
    assert_cell(cell);
    if !has(set, cell) {
        set + (1 << cell) + (1 << SIZE_SHIFT)
    } else {
        set
    }
}

pub fn remove(set: &mut CellSet, cell: Cell) {
    assert_cell(cell);
    if has(*set, cell) {
        *set &= !(1 << cell);
        *set -= 1 << SIZE_SHIFT;
    }
}

pub const fn without(set: CellSet, cell: Cell) -> CellSet {
    assert_cell(cell);
    if has(set, cell) {
        set - (1 << cell) - (1 << SIZE_SHIFT)
    } else {
        set
    }
}

pub fn invert(set: &mut CellSet) {
    *set = inverted(*set)
}

pub const fn inverted(set: CellSet) -> CellSet {
    pack(!set & BITS_MASK, CELL_COUNT - size(set))
}

pub const fn union(set1: CellSet, set2: CellSet) -> CellSet {
    new((set1 | set2) & BITS_MASK)
}

pub const fn intersect(set1: CellSet, set2: CellSet) -> CellSet {
    new(set1 & set2 & BITS_MASK)
}

pub const fn diff(set1: CellSet, set2: CellSet) -> CellSet {
    new(set1 & !set2 & BITS_MASK)
}

pub fn debug(set: CellSet) -> String {
    format!("{:02}:{:081b}", size(set), bits(set))
}

pub fn to_string(set: CellSet) -> String {
    if is_empty(set) {
        return EMPTY.to_string();
    }

    let mut s = String::with_capacity(3 * size(set) as usize + 1);
    let mut first = true;
    s.push('(');
    for i in ALL_CELLS {
        if has(set, i) {
            if first {
                first = false;
            } else {
                s.push(' ');
            }
            s.push_str(label_from_cell(i));
        }
    }
    s.push(')');
    s
}

const EMPTY: &str = "∅";

pub const fn row_from_cell(cell: Cell) -> Row {
    (cell / 9) as Row
}

pub const fn cell_in_row(row: Row, col: Column) -> Cell {
    (row * 9 + col) as Cell
}

pub const fn column_from_cell(cell: Cell) -> Column {
    (cell % 9) as Column
}

pub const fn cell_in_column(col: Column, row: Row) -> Cell {
    (row * 9 + col) as Cell
}

pub const fn block_from_cell(cell: Cell) -> Block {
    (3 * (cell / 27) + (cell % 9) / 3) as Block
}

pub const fn cell_in_block(block: Block, coord: Coord) -> Cell {
    (27 * (block / 3) + 3 * (block % 3) + 9 * (coord / 3) + (coord % 3)) as Cell
}

pub const fn label_from_cell(cell: Cell) -> &'static str {
    CELL_LABELS[cell as usize]
}

pub fn cell_from_label(label: &str) -> Cell {
    if label.len() != 2 {
        panic!("Invalid cell: {}", label);
    }
    let mut chars = label.chars();
    let row = chars.next().unwrap() as Cell - b'A';
    let col = chars.next().unwrap() as Cell - b'1';
    // allow both I and J for 9th row and handle below
    if row > 9 || col >= 9 {
        panic!("Invalid cell: {}", label);
    }
    if row == 9 {
        8 * 9 + col
    } else {
        row * 9 + col
    }
}

#[rustfmt::skip]
const CELL_LABELS: [&str; 81] = [
    "A1", "A2", "A3", "A4", "A5", "A6", "A7", "A8", "A9",
    "B1", "B2", "B3", "B4", "B5", "B6", "B7", "B8", "B9",
    "C1", "C2", "C3", "C4", "C5", "C6", "C7", "C8", "C9",
    "D1", "D2", "D3", "D4", "D5", "D6", "D7", "D8", "D9",
    "E1", "E2", "E3", "E4", "E5", "E6", "E7", "E8", "E9",
    "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9",
    "G1", "G2", "G3", "G4", "G5", "G6", "G7", "G8", "G9",
    "H1", "H2", "H3", "H4", "H5", "H6", "H7", "H8", "H9",
    "J1", "J2", "J3", "J4", "J5", "J6", "J7", "J8", "J9",
];
#[rustfmt::skip]
const CELL_LABELS_WITH_SPACES: [&str; 81] = [
    " A1", " A2", " A3", " A4", " A5", " A6", " A7", " A8", " A9",
    " B1", " B2", " B3", " B4", " B5", " B6", " B7", " B8", " B9",
    " C1", " C2", " C3", " C4", " C5", " C6", " C7", " C8", " C9",
    " D1", " D2", " D3", " D4", " D5", " D6", " D7", " D8", " D9",
    " E1", " E2", " E3", " E4", " E5", " E6", " E7", " E8", " E9",
    " F1", " F2", " F3", " F4", " F5", " F6", " F7", " F8", " F9",
    " G1", " G2", " G3", " G4", " G5", " G6", " G7", " G8", " G9",
    " H1", " H2", " H3", " H4", " H5", " H6", " H7", " H8", " H9",
    " J1", " J2", " J3", " J4", " J5", " J6", " J7", " J8", " J9",
];
// const ROW_COORDS: &str = "ABCDEFGHJ";
// const ROW_COORDS: [char; 9] = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J'];
// const COLUMN_COORDS: [char; 9] = ['1', '2', '3', '4', '5', '6', ''B2'', '8', '9'];

pub const fn count_bits(mut bits: Bits) -> Size {
    let mut count: Size = 0;
    while bits != 0 {
        count += NIBBLE_COUNTS[(bits & 0b1111) as usize];
        bits >>= 4;
    }
    count
}

const NIBBLE_COUNTS: [Size; 16] = [0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4];

#[cfg(test)]
mod tests {
    use super::*;
    use rand::prelude::*;

    #[test]
    fn empty_returns_an_empty_set() {
        let set = empty();

        assert!(is_empty(set));
        assert_eq!(size(set), 0);
        for i in ALL_CELLS {
            assert!(!has(set, i));
        }
    }

    #[test]
    fn full_returns_a_full_set() {
        let set = full();

        assert!(!is_empty(set));
        assert_eq!(size(set), CELL_COUNT);
        for i in ALL_CELLS {
            assert!(has(set, i));
        }
    }

    #[test]
    fn of_cells_returns_a_set() {
        let mut rng = thread_rng();
        let mut cells = Vec::with_capacity(81);

        for _ in 0..81 {
            cells.push(rng.gen_range(ALL_CELLS));
        }
        let set = of_cells(&cells);

        assert!(!is_empty(set));
        for i in 0..81 {
            assert!(has(set, cells[i]));
        }
    }

    #[test]
    fn of_returns_a_set() {
        let mut rng = thread_rng();
        let mut labels = Vec::with_capacity(81);

        for _ in 0..81 {
            labels.push(label_from_cell(rng.gen_range(0..81)));
        }
        let set = of(&labels);

        assert!(!is_empty(set));
        for i in 0..81 {
            assert!(has(set, cell_from_label(labels[i])));
        }
    }

    #[test]
    fn inverting() {
        let mut rng = thread_rng();
        let mut set = empty();

        for _ in 0..81 {
            let cell = rng.gen_range(ALL_CELLS);
            add(&mut set, cell);
        }

        let inverted = inverted(set);
        for i in ALL_CELLS {
            assert_eq!(has(set, i), !has(inverted, i));
        }
    }

    #[test]
    fn add_cells() {
        let mut rng = thread_rng();
        let mut set = empty();

        for _ in 0..81 {
            let cell = rng.gen_range(ALL_CELLS);
            let s = size(set);
            let h = has(set, cell);

            add(&mut set, cell);
            assert!(has(set, cell));
            if h {
                assert_eq!(size(set), s);
            } else {
                assert_eq!(size(set), s + 1);
            }
        }
    }

    #[test]
    fn with_cells() {
        let mut rng = thread_rng();
        let mut set = empty();

        for _ in 0..81 {
            let cell = rng.gen_range(ALL_CELLS);
            let added = with(set, cell);

            assert!(has(added, cell));
            if has(set, cell) {
                assert_eq!(size(added), size(set));
            } else {
                assert_eq!(size(added), size(set) + 1);
            }

            set = added;
        }
    }

    #[test]
    fn remove_cells() {
        let mut rng = thread_rng();
        let mut set = full();

        for _ in 0..81 {
            let cell = rng.gen_range(ALL_CELLS);
            let s = size(set);
            let h = has(set, cell);

            remove(&mut set, cell);
            assert!(!has(set, cell));
            if h {
                assert_eq!(size(set), s - 1);
            } else {
                assert_eq!(size(set), s);
            }
        }
    }

    #[test]
    fn without_cells() {
        let mut rng = thread_rng();
        let mut set = full();

        for _ in 0..81 {
            let cell = rng.gen_range(ALL_CELLS);
            let removed = without(set, cell);

            assert!(!has(removed, cell));
            if has(set, cell) {
                assert_eq!(size(removed), size(set) - 1);
            } else {
                assert_eq!(size(removed), size(set));
            }

            set = removed;
        }
    }

    #[test]
    fn unions() {
        assert_eq!(union(empty(), empty()), empty());
        assert_eq!(union(full(), full()), full());
        assert_eq!(union(full(), empty()), full());
        assert_eq!(union(empty(), full()), full());

        assert_eq!(
            union(of(&["B2", "F3"]), of(&["C6", "H8"])),
            of(&["B2", "C6", "F3", "H8"])
        );
        assert_eq!(
            union(of(&["B2", "C6", "F3"]), of(&["C6", "H8"])),
            of(&["B2", "C6", "F3", "H8"])
        );
    }

    #[test]
    fn intersections() {
        assert_eq!(intersect(empty(), empty()), empty());
        assert_eq!(intersect(full(), full()), full());
        assert_eq!(intersect(full(), empty()), empty());
        assert_eq!(intersect(empty(), full()), empty());

        assert_eq!(
            intersect(of(&["B2", "C6", "F3"]), of(&["C6", "F3", "H8"])),
            of(&["C6", "F3"])
        );
    }

    #[test]
    fn differences() {
        assert_eq!(diff(empty(), empty()), empty());
        assert_eq!(diff(full(), full()), empty());
        assert_eq!(diff(full(), empty()), full());
        assert_eq!(diff(empty(), full()), empty());

        assert_eq!(
            diff(of(&["B2", "C6", "F3", "H8"]), of(&["C6", "H8"])),
            of(&["B2", "F3"])
        );
        assert_eq!(
            diff(of(&["B2", "C6", "F3", "H8"]), of(&["A5", "C6", "F9", "H8"])),
            of(&["B2", "F3"])
        );
    }

    #[test]
    fn debug_strings() {
        assert_eq!(
            debug(empty()),
            "00:000000000000000000000000000000000000000000000000000000000000000000000000000000000"
        );
        assert_eq!(
            debug(full()),
            "81:111111111111111111111111111111111111111111111111111111111111111111111111111111111"
        );
        assert_eq!(
            debug(of(&["B2", "C6", "F3", "H8"])),
            "04:000000000010000000000000000000000100000000000000000000000100000000000010000000000"
        );
    }

    #[test]
    fn to_string_returns_cell_coords() {
        assert_eq!(to_string(empty()), EMPTY);
        assert_eq!(to_string(of(&["B2", "C6", "F3", "H8"])), "(B2 C6 F3 H8)");
    }
}
