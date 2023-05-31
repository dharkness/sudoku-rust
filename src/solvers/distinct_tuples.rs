pub fn distinct_pairs<T: Copy>(items: &Vec<T>) -> Vec<(T, T)> {
    let mut pairs = Vec::new();
    if items.len() >= 2 {
        for i in 0..(items.len() - 1) {
            for j in i + 1..items.len() {
                pairs.push((items[i], items[j]));
            }
        }
    }
    pairs
}

pub fn distinct_triples<T: Copy>(items: &Vec<T>) -> Vec<(T, T, T)> {
    let mut pairs = Vec::new();
    if items.len() >= 3 {
        for i in 0..(items.len() - 2) {
            for j in i + 1..(items.len() - 1) {
                for k in j + 1..items.len() {
                    pairs.push((items[i], items[j], items[k]));
                }
            }
        }
    }
    pairs
}

pub fn distinct_quads<T: Copy>(items: &Vec<T>) -> Vec<(T, T, T, T)> {
    let mut pairs = Vec::new();
    if items.len() >= 4 {
        for i in 0..(items.len() - 3) {
            for j in i + 1..(items.len() - 2) {
                for k in j + 1..(items.len() - 1) {
                    for l in k + 1..items.len() {
                        pairs.push((items[i], items[j], items[k], items[l]));
                    }
                }
            }
        }
    }
    pairs
}
