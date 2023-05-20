use crate::layout::{Cell, CellSet};

pub fn generate_code_for_neighbors() {
    println!("const NEIGHBORS: [Set; 81] = [");
    for i in 0..81 {
        let mut neighbors = CellSet::empty();
        let cell = Cell::new(i);

        neighbors |= cell.row().cells();
        neighbors |= cell.column().cells();
        neighbors |= cell.block().cells();
        neighbors -= cell;

        println!("    Set::new(0b{:081b}),", neighbors.bits());
    }
    println!("];");
}
