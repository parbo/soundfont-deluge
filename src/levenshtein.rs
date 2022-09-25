// From https://en.wikipedia.org/wiki/Levenshtein_distance
pub fn distance(s: &str, t: &str) -> usize {
    let n = t.len();
    // create two work vectors of integer distances
    // initialize v0 (the previous row of distances)
    // this row is A[0][i]: edit distance from an empty s to t;
    // that distance is the number of characters to append to  s to make t.
    let mut v0: Vec<usize> = (0..(n + 1)).into_iter().collect();
    let mut v1: Vec<usize> = vec![0; n + 1];

    for (i, sc) in s.chars().enumerate() {
        // calculate v1 (current row distances) from the previous row v0

        // first element of v1 is A[i + 1][0]
        //   edit distance is delete (i + 1) chars from s to match empty t
        v1[0] = i + 1;

        // use formula to fill in the rest of the row
        for (j, tc) in t.chars().enumerate() {
            // calculating costs for A[i + 1][j + 1]
            let deletion_cost = v0[j + 1] + 1;
            let insertion_cost = v1[j] + 1;
            let substitution_cost = if sc == tc { v0[j] } else { v0[j] + 1 };

            v1[j + 1] = [deletion_cost, insertion_cost, substitution_cost]
                .into_iter()
                .min()
                .unwrap();
        }

        // copy v1 (current row) to v0 (previous row) for next iteration
        // since data in v1 is always invalidated, a swap without copy could be more efficient
        std::mem::swap(&mut v0, &mut v1);
    }
    // after the last swap, the results of v1 are now in v0
    v0[n]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance() {
        let data = [
            ("water", "wine", 3),
            ("kitten", "sitting", 3),
            ("saturday", "sunday", 3),
            ("pheromones", "photographer", 8),
        ];
        for (a, b, d) in data {
            assert_eq!(distance(a, b), d);
        }
    }
}
