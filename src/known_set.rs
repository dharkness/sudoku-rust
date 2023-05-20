pub type Value = u8; // includes UNKNOWN
pub type Known = u8;
pub type KnownSet = u16;

type Bits = u16;
type Size = u8;

pub const UNKNOWN: Value = 0;
pub const ALL_KNOWNS: std::ops::RangeInclusive<Known> = 1..=KNOWN_COUNT;

const KNOWN_COUNT: Size = 9;
const ALL_BITS_SET: Bits = (1 << KNOWN_COUNT) - 1;

const BITS_MASK: Bits = (1 << KNOWN_COUNT) - 1;
const SIZE_SHIFT: usize = 16 - 4;
const MARKER: Bits = 0b0000110000000000;

pub fn assert_known(known: Known) {
    assert!(ALL_KNOWNS.contains(&known));
}

fn pack(bits: Bits, size: Size) -> KnownSet {
    assert_eq!(bits, bits & BITS_MASK);
    assert!(size <= KNOWN_COUNT);
    ((size as Bits) << SIZE_SHIFT) + bits
}

pub fn empty() -> KnownSet {
    0
}

pub fn full() -> KnownSet {
    pack(ALL_BITS_SET, KNOWN_COUNT)
}

pub fn new(bits: Bits) -> KnownSet {
    pack(bits, count_bits(bits))
}

pub fn of(knowns: &[Known]) -> KnownSet {
    let mut set = empty();
    for known in knowns {
        add(&mut set, *known);
    }
    set
}

pub fn is_empty(set: KnownSet) -> bool {
    set == 0
}

pub fn size(set: KnownSet) -> Size {
    (set >> SIZE_SHIFT) as Size
}

pub fn bits(set: KnownSet) -> Bits {
    set & BITS_MASK
}

pub fn has(set: KnownSet, known: Known) -> bool {
    assert_known(known);
    set & (1 << (known - 1)) != 0
}

pub fn invert(set: &mut KnownSet) {
    *set = inverted(*set)
}

pub fn inverted(set: KnownSet) -> KnownSet {
    pack(!bits(set) & BITS_MASK, KNOWN_COUNT - size(set))
}

pub fn add(set: &mut KnownSet, known: Known) {
    assert_known(known);
    if !has(*set, known) {
        *set |= 1 << (known - 1);
        *set += 1 << SIZE_SHIFT;
    }
}

pub fn with(set: KnownSet, known: Known) -> KnownSet {
    assert_known(known);
    if !has(set, known) {
        set + (1 << (known - 1)) + (1 << SIZE_SHIFT)
    } else {
        set
    }
}

pub fn remove(set: &mut KnownSet, known: Known) {
    assert_known(known);
    if has(*set, known) {
        *set &= !(1 << (known - 1));
        *set -= 1 << SIZE_SHIFT;
    }
}

pub fn without(set: KnownSet, known: Known) -> KnownSet {
    assert_known(known);
    if has(set, known) {
        set - (1 << (known - 1)) - (1 << SIZE_SHIFT)
    } else {
        set
    }
}

pub fn union(set1: KnownSet, set2: KnownSet) -> KnownSet {
    new((set1 | set2) & BITS_MASK)
}

pub fn intersect(set1: KnownSet, set2: KnownSet) -> KnownSet {
    new(set1 & set2 & BITS_MASK)
}

pub fn diff(set1: KnownSet, set2: KnownSet) -> KnownSet {
    new(set1 & !set2 & BITS_MASK)
}

pub fn debug(set: KnownSet) -> String {
    format!("{:01}:{:09b}", size(set), bits(set))
}

pub fn to_string(set: KnownSet) -> String {
    if is_empty(set) {
        return EMPTY.to_string();
    }

    let mut s = String::with_capacity(2 * KNOWN_COUNT as usize + 1);
    s.push('(');
    for i in ALL_KNOWNS {
        if has(set, i) {
            s.push((b'0' + i) as char);
        } else {
            s.push(MISSING);
        }
    }
    s.push(')');
    s
}

const EMPTY: char = '∅';
const MISSING: char = '·';

fn count_bits(bits: Bits) -> Size {
    NIBBLE_COUNTS[(bits & 0b1111) as usize]
        + NIBBLE_COUNTS[((bits >> 4) & 0b1111) as usize]
        + NIBBLE_COUNTS[((bits >> 8) & 0b1111) as usize]
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
        for i in ALL_KNOWNS {
            assert!(!has(set, i));
        }
    }

    #[test]
    fn full_returns_a_full_set() {
        let set = full();

        assert!(!is_empty(set));
        assert_eq!(size(set), KNOWN_COUNT);
        for i in ALL_KNOWNS {
            assert!(has(set, i));
        }
    }

    #[test]
    fn of_returns_a_set() {
        let mut rng = thread_rng();
        let mut knowns = Vec::with_capacity(9);

        for _ in 0..9 {
            knowns.push(rng.gen_range(ALL_KNOWNS));
        }
        let set = of(&knowns);

        assert!(!is_empty(set));
        for i in 0..9 {
            assert!(has(set, knowns[i]));
        }
    }

    #[test]
    fn inverting() {
        let mut rng = thread_rng();
        let mut set = empty();

        for _ in 0..9 {
            let known = rng.gen_range(ALL_KNOWNS);
            add(&mut set, known);
        }

        let inverted = inverted(set);
        for i in ALL_KNOWNS {
            assert_eq!(has(set, i), !has(inverted, i));
        }
    }

    #[test]
    fn add_knowns() {
        let mut rng = thread_rng();
        let mut set = empty();

        for _ in 0..9 {
            let known = rng.gen_range(ALL_KNOWNS);
            let s = size(set);
            let h = has(set, known);

            add(&mut set, known);
            assert!(has(set, known));
            if h {
                assert_eq!(size(set), s);
            } else {
                assert_eq!(size(set), s + 1);
            }
        }
    }

    #[test]
    fn with_knowns() {
        let mut rng = thread_rng();
        let mut set = empty();

        for _ in 0..9 {
            let known = rng.gen_range(ALL_KNOWNS);
            let added = with(set, known);

            assert!(has(added, known));
            if has(set, known) {
                assert_eq!(size(added), size(set));
            } else {
                assert_eq!(size(added), size(set) + 1);
            }

            set = added;
        }
    }

    #[test]
    fn remove_knowns() {
        let mut rng = thread_rng();
        let mut set = full();

        for _ in 0..9 {
            let known = rng.gen_range(ALL_KNOWNS);
            let s = size(set);
            let h = has(set, known);

            remove(&mut set, known);
            assert!(!has(set, known));
            if h {
                assert_eq!(size(set), s - 1);
            } else {
                assert_eq!(size(set), s);
            }
        }
    }

    #[test]
    fn without_knowns() {
        let mut rng = thread_rng();
        let mut set = full();

        for _ in 0..9 {
            let known = rng.gen_range(ALL_KNOWNS);
            let removed = without(set, known);

            assert!(!has(removed, known));
            if has(set, known) {
                assert_eq!(size(removed), size(set) - 1);
            } else {
                assert_eq!(size(removed), size(set));
            }

            set = removed;
        }
    }

    #[test]
    fn unions() {
        println!("{}", debug(empty()));
        println!("{}", debug(full()));
        println!("{}", debug(union(full(), full())));

        assert_eq!(union(empty(), empty()), empty());
        assert_eq!(union(full(), full()), full());
        assert_eq!(union(full(), empty()), full());
        assert_eq!(union(empty(), full()), full());

        assert_eq!(union(of(&[2, 5]), of(&[3, 8])), of(&[2, 3, 5, 8]));
        assert_eq!(union(of(&[2, 3, 5]), of(&[3, 8])), of(&[2, 3, 5, 8]));
    }

    #[test]
    fn intersections() {
        assert_eq!(intersect(empty(), empty()), empty());
        assert_eq!(intersect(full(), full()), full());
        assert_eq!(intersect(full(), empty()), empty());
        assert_eq!(intersect(empty(), full()), empty());

        assert_eq!(intersect(of(&[2, 3, 5]), of(&[3, 5, 8])), of(&[3, 5]));
    }

    #[test]
    fn differences() {
        assert_eq!(diff(empty(), empty()), empty());
        assert_eq!(diff(full(), full()), empty());
        assert_eq!(diff(full(), empty()), full());
        assert_eq!(diff(empty(), full()), empty());

        assert_eq!(diff(of(&[2, 3, 5, 8]), of(&[3, 8])), of(&[2, 5]));
        assert_eq!(diff(of(&[2, 3, 5, 8]), of(&[1, 3, 6, 8])), of(&[2, 5]));
    }

    #[test]
    fn debug_strings() {
        assert_eq!(debug(empty()), "0:000000000");
        assert_eq!(debug(full()), "9:111111111");
        assert_eq!(debug(of(&[2, 3, 5, 8])), "4:010010110");
    }

    #[test]
    fn to_string_returns_cell_knowns() {
        assert_eq!(to_string(empty()), EMPTY.to_string());
        assert_eq!(to_string(of(&[2, 3, 5, 8])), "(·23·5··8·)");
    }
}
