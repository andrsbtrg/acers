use std::collections::HashSet;

pub fn unique_pairs<T: Clone + Eq + std::hash::Hash + std::cmp::PartialOrd>(
    items: &[T],
) -> Vec<(T, T)> {
    let mut seen = HashSet::new();
    let mut pairs = Vec::new();

    for (i, a) in items.iter().enumerate() {
        for b in &items[i + 1..] {
            let pair = (a.clone(), b.clone());

            // Use a tuple with sorted elements to ensure uniqueness
            let sorted_pair = if pair.0 < pair.1 {
                pair.clone()
            } else {
                (pair.1.clone(), pair.0.clone())
            };

            if seen.insert(sorted_pair.clone()) {
                pairs.push(sorted_pair);
            }
        }
    }

    pairs
}
