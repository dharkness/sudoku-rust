use std::fmt;

#[derive(Debug, Clone)]
pub struct CoordSet {
    bits: i16,
    size: i16,
}

impl CoordSet {
    pub fn empty() -> CoordSet {
        CoordSet { bits: 0, size: 0 }
    }

    pub fn full() -> CoordSet {
        CoordSet {
            bits: 0b111111111,
            size: 9,
        }
    }

    pub fn of(coords: &[i16]) -> CoordSet {
        let mut set = CoordSet::empty();
        for coord in coords {
            set.add(*coord);
        }
        set
    }

    pub fn new(bits: i16) -> CoordSet {
        CoordSet {
            bits,
            size: coord_set_size(bits),
        }
    }

    pub fn size(&self) -> i16 {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn has(&self, coord: i16) -> bool {
        assert_coord(coord);
        let bit = 1 << coord;
        self.bits & bit != 0
    }

    pub fn add(&mut self, coord: i16) {
        assert_coord(coord);
        let bit = 1 << coord;
        if self.bits & bit == 0 {
            self.bits += bit;
            self.size += 1;
        }
    }

    pub fn remove(&mut self, coord: i16) {
        assert_coord(coord);
        let bit = 1 << coord;
        if self.bits & bit != 0 {
            self.bits -= bit;
            self.size -= 1;
        }
    }

    pub fn union(&self, other: &CoordSet) -> CoordSet {
        CoordSet::new(self.bits | other.bits)
    }

    pub fn intersect(&self, other: &CoordSet) -> CoordSet {
        CoordSet::new(self.bits & other.bits)
    }

    pub fn difference(&self, other: &CoordSet) -> CoordSet {
        CoordSet::new(self.bits & -other.bits)
    }

    pub fn to_coord_string(&self) -> String {
        let mut coords = String::new();
        for bit in 0..9 {
            if self.has(bit) {
                coords.push((b'1' + bit as u8) as char);
            } else {
                coords.push('·');
            }
        }
        coords
    }

    pub fn debug(&self) {
        println!("coords: {:09b}", self.bits);
        println!("size: {}", self.size);
    }
}

impl fmt::Display for CoordSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_coord_string())
    }
}

pub fn coord_set_size(bits: i16) -> i16 {
    COORD_SET_SIZES[bits as usize] as i16
}

const COORD_SET_SIZES: [i8; 1 << 9] = [
    0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4, 1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5,
    1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5, 2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6,
    1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5, 2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6,
    2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6, 3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7,
    1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5, 2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6,
    2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6, 3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7,
    2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6, 3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7,
    3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7, 4, 5, 5, 6, 5, 6, 6, 7, 5, 6, 6, 7, 6, 7, 7, 8,
    1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5, 2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6,
    2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6, 3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7,
    2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6, 3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7,
    3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7, 4, 5, 5, 6, 5, 6, 6, 7, 5, 6, 6, 7, 6, 7, 7, 8,
    2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6, 3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7,
    3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7, 4, 5, 5, 6, 5, 6, 6, 7, 5, 6, 6, 7, 6, 7, 7, 8,
    3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7, 4, 5, 5, 6, 5, 6, 6, 7, 5, 6, 6, 7, 6, 7, 7, 8,
    4, 5, 5, 6, 5, 6, 6, 7, 5, 6, 6, 7, 6, 7, 7, 8, 5, 6, 6, 7, 6, 7, 7, 8, 6, 7, 7, 8, 7, 8, 8, 9,
];

pub fn assert_coord(coord: i16) {
    assert!((0..9).contains(&coord));
}

pub fn generate_code_for_coord_set_sizes() {
    let mut table: Vec<i16> = vec![0; 1 << 9];

    for (bits, size) in table.iter_mut().enumerate().take(1 << 9) {
        *size = bits.count_ones() as i16;
    }

    println!("const COORD_SET_SIZES: [i16; 1 << 9] = [");
    for (bits, size) in table.iter().enumerate().take(1 << 9) {
        if bits % 8 == 0 {
            print!("    ");
        }
        print!("{}, ", size);
        if bits % 8 == 7 {
            println!();
        }
    }
    println!("];");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let set = CoordSet::empty();

        assert!(set.is_empty());
        assert_eq!(set.size(), 0);
        for i in 0..9 {
            assert!(!set.has(i));
        }
        assert_eq!(set.to_coord_string(), "·········");
    }

    #[test]
    fn full() {
        let set = CoordSet::full();

        assert!(!set.is_empty());
        assert_eq!(set.size(), 9);
        for i in 0..9 {
            assert!(set.has(i));
        }
        assert_eq!(set.to_coord_string(), "123456789");
    }

    #[test]
    fn add() {
        let mut set = CoordSet::of(&[2, 4, 6]);

        set.add(2);
        assert!(set.has(2));
        assert_eq!(set.size(), 3);

        set.add(3);
        assert!(set.has(3));
        assert_eq!(set.size(), 4);
    }

    #[test]
    fn remove() {
        let mut set = CoordSet::of(&[2, 4, 6]);

        set.remove(3);
        assert!(set.has(2));
        assert!(!set.has(3));
        assert_eq!(set.size(), 3);

        set.remove(2);
        assert!(!set.has(2));
        assert_eq!(set.size(), 2);
    }

    #[test]
    fn clone() {
        let mut set = CoordSet::of(&[2, 4, 6]);
        let mut other = set.clone();

        set.add(3);
        assert!(set.has(3));
        assert_eq!(set.size(), 4);
        assert!(!other.has(3));
        assert_eq!(other.size(), 3);

        other.remove(4);
        assert!(!other.has(4));
        assert_eq!(other.size(), 2);
        assert!(set.has(4));
        assert_eq!(set.size(), 4);
    }
}
