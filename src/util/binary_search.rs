/// Returns the index of the element in `values` that is numerically closest to `target`. On a tie (two values equally close) the lower index is chosen.
pub fn closest_index(values: &[usize], target: usize) -> Option<usize> {
    if values.is_empty() {
        return None;
    }

    match values.binary_search(&target) {
        // Exact hit ─ best possible.
        Ok(idx) => Some(idx),

        // `Err(pos)` gives the index where `target` could be inserted to keep the order.
        Err(pos) => {
            match pos {
                // target < first element
                0 => Some(0),
                // target > last element
                n if n == values.len() => Some(values.len() - 1),
                // Compare distances to the two neighbors: values[pos-1] and values[pos].
                _ => {
                    let diff_low = target - values[pos - 1];
                    let diff_high = values[pos] - target;
                    if diff_low <= diff_high {
                        Some(pos - 1)
                    } else {
                        Some(pos)
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::util::binary_search::closest_index;

    #[test]
    fn closest_within_range() {
        // 8 is the closest value to 10 → index 2
        let v = vec![0, 3, 8, 12, 20];
        assert_eq!(closest_index(&v, 10), Some(2));
    }

    #[test]
    fn closest_exact_match() {
        // Exact hit: 12 is at index 3
        let v = vec![0, 3, 8, 12, 20];
        assert_eq!(closest_index(&v, 12), Some(3));
    }

    #[test]
    fn closest_below_range() {
        // Target is smaller than the first element → choose index 0
        let v = vec![0, 3, 8, 12, 20];
        assert_eq!(closest_index(&v, 1), Some(0));
    }

    #[test]
    fn closest_above_range() {
        // Target is larger than the last element → choose last index (4)
        let v = vec![0, 3, 8, 12, 20];
        assert_eq!(closest_index(&v, 25), Some(4));
    }
}
