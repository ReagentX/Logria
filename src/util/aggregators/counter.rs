use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
};

use crate::{
    constants::cli::colors::RESET_COLOR,
    util::{
        aggregators::aggregator::{Aggregator, format_int},
        error::LogriaError,
    },
};

/// A counter for tracking occurrences of messages, similar to Python's `Counter`.
pub struct Counter {
    /// Map of message strings to their occurrence counts.
    counts: HashMap<String, u64>,
    /// Total number of messages processed.
    total_count: u64,
    /// Optional limit on the number of top messages to display.
    frozen_n: Option<usize>,
}

impl Aggregator for Counter {
    /// Implements [`Aggregator::update`]: increments the count for the given message.
    fn update(&mut self, message: &str) -> Result<(), LogriaError> {
        self.increment(message);
        Ok(())
    }

    /// Implements [`Aggregator::messages`]: returns the top `n` messages.
    fn messages(&self, n: &usize) -> Vec<String> {
        self.get_top_messages(*n)
    }

    /// Implements [`Aggregator::reset`]: clears all message counts.
    fn reset(&mut self) {
        self.counts.clear();
        self.total_count = 0;
    }
}

impl Counter {
    /// Creates a new `Counter` with no messages counted and no limits.
    pub fn new() -> Counter {
        Counter {
            counts: HashMap::new(),
            total_count: 0,
            frozen_n: None,
        }
    }

    /// Creates a new `Counter` configured to return only the top message.
    pub fn mean() -> Counter {
        Counter {
            counts: HashMap::new(),
            total_count: 0,
            frozen_n: Some(1),
        }
    }

    /// Increments the count for `item`, adding it if not already present.
    pub fn increment(&mut self, item: &str) {
        let count = self.counts.entry(item.to_string()).or_insert(0);
        *count += 1;
        self.total_count += 1;
    }

    /// Decrements the count for `item`, removing it if its count reaches zero.
    pub fn decrement(&mut self, item: &str) {
        if let Some(count) = self.counts.get_mut(item) {
            if *count > 1 {
                *count -= 1;
                self.total_count -= 1;
            } else {
                self.counts.remove(item);
                self.total_count -= 1;
            }
        }
    }

    /// Removes `item` entirely from the counter, subtracting its count from the total.
    pub fn delete(&mut self, item: &str) {
        if let Some(count) = self.counts.remove(item) {
            self.total_count -= count;
        }
    }

    /// Retrieves the top `n` messages, respecting `frozen_n` if set.
    fn get_top_messages(&self, n: usize) -> Vec<String> {
        let actual_num = self.frozen_n.unwrap_or(n).min(self.counts.len());

        if actual_num == 0 || self.total_count == 0 {
            return Vec::new();
        }

        self.compute_top_messages(actual_num)
    }

    /// Computes the top `n` messages sorted by count (descending) and message text.
    fn compute_top_messages(&self, n: usize) -> Vec<String> {
        // A min-heap that only ever holds the top n entries.
        let total = self.total_count as f64;
        let mut heap = BinaryHeap::with_capacity(n + 1);

        for (item, &count) in &self.counts {
            // Reverse so that smallest count is at the top—and will get popped when > n
            heap.push(Reverse((count, item.as_str())));
            if heap.len() > n {
                heap.pop();
            }
        }

        // Drain heap into a Vec, sort descending by count then key
        let mut top: Vec<(u64, &str)> = heap
            .into_iter()
            .map(|Reverse((count, item))| (count, item))
            .collect();

        top.sort_unstable_by(|(ca, a), (cb, b)| cb.cmp(ca).then_with(|| a.cmp(b)));

        // Format output
        top.into_iter()
            .map(|(count, item)| {
                let pct = (count as f64 / total) * 100.0;
                format!(
                    "    {}{}: {} ({:.0}%)",
                    item.trim(),
                    RESET_COLOR,
                    format_int(count as usize),
                    pct
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod behavior_tests {
    use crate::util::aggregators::{aggregator::Aggregator, counter::Counter};
    use std::collections::HashMap;

    static A: &str = "a";
    static B: &str = "b";

    #[test]
    fn can_construct_counter() {
        Counter::new();
    }

    #[test]
    fn can_count_int() {
        let mut c: Counter = Counter::new();
        c.increment("1");
        c.increment("1");
        c.increment("1");
        c.increment("2");
        c.increment("2");

        let mut expected_count = HashMap::new();
        expected_count.insert("1".to_string(), 3);
        expected_count.insert("2".to_string(), 2);

        assert_eq!(c.counts.get("1"), Some(&3));
        assert_eq!(c.counts.get("2"), Some(&2));
        assert_eq!(c.total_count, 5);
    }

    #[test]
    fn can_count() {
        let mut c: Counter = Counter::new();
        c.increment(A);
        c.increment(A);
        c.increment(A);
        c.increment(B);
        c.increment(B);

        let mut expected_count = HashMap::new();
        expected_count.insert(A.to_owned(), 3);
        expected_count.insert(B.to_owned(), 2);

        assert_eq!(c.counts.get(A), Some(&3));
        assert_eq!(c.counts.get(B), Some(&2));
        assert_eq!(c.total_count, 5);
    }

    #[test]
    fn can_sum() {
        let mut c: Counter = Counter::new();
        c.update(A).unwrap();
        c.update(A).unwrap();
        c.update(A).unwrap();
        c.update(B).unwrap();
        c.update(B).unwrap();

        let mut expected = HashMap::new();
        expected.insert(A.to_owned(), 3);
        expected.insert(B.to_owned(), 2);

        assert_eq!(c.total_count, 5);
    }

    #[test]
    fn can_decrement() {
        let mut c: Counter = Counter::new();
        c.increment(A);
        c.increment(A);
        c.increment(A);
        c.increment(B);
        c.increment(B);
        c.decrement(A);

        let mut expected_count = HashMap::new();
        expected_count.insert(A.to_owned(), 2);
        expected_count.insert(B.to_owned(), 2);

        assert_eq!(c.counts.get(A), Some(&2));
        assert_eq!(c.counts.get(B), Some(&2));
        assert_eq!(c.total_count, 4);
    }

    #[test]
    fn can_decrement_auto_remove() {
        let mut c: Counter = Counter::new();
        c.increment(A);
        c.increment(B);
        c.increment(B);
        c.decrement(A);

        let mut expected_count = HashMap::new();
        expected_count.insert(B.to_owned(), 2);

        assert_eq!(c.counts.get(B), Some(&2));
        assert_eq!(c.counts.get(A), None);
        assert_eq!(c.total_count, 2);
    }

    #[test]
    fn can_delete() {
        let mut c: Counter = Counter::new();
        c.increment(A);
        c.increment(A);
        c.increment(A);
        c.increment(B);
        c.increment(B);
        c.delete(A);

        assert_eq!(c.counts.get(B), Some(&2));
        assert_eq!(c.counts.get(A), None);
        assert_eq!(c.total_count, 2);
    }
}

#[cfg(test)]
mod message_tests {
    use crate::util::aggregators::{aggregator::Aggregator, counter::Counter};

    static A: &str = "a";
    static B: &str = "b";
    static C: &str = "c";
    static D: &str = "d";

    #[test]
    fn can_get_top_0() {
        let mut c: Counter = Counter::new();
        c.increment(A);
        c.increment(A);
        c.increment(A);
        c.increment(B);
        c.increment(B);
        c.increment(B);
        c.increment(C);
        c.increment(C);
        c.increment(D);

        let expected: Vec<String> = vec![];

        assert_eq!(c.messages(&0), expected);
    }

    #[test]
    fn can_get_top_1() {
        let mut c: Counter = Counter::new();
        c.increment(A);
        c.increment(A);
        c.increment(A);
        c.increment(A);
        c.increment(B);
        c.increment(B);
        c.increment(B);
        c.increment(C);
        c.increment(C);
        c.increment(D);

        let expected = vec![String::from("    a\u{1b}[0m: 4 (40%)")];

        assert_eq!(c.messages(&1), expected);
    }

    #[test]
    fn can_get_top_2() {
        let mut c: Counter = Counter::new();
        c.increment(A);
        c.increment(A);
        c.increment(A);
        c.increment(B);
        c.increment(B);
        c.increment(B);
        c.increment(C);
        c.increment(C);
        c.increment(D);

        let expected = vec![
            String::from("    a\u{1b}[0m: 3 (33%)"),
            String::from("    b\u{1b}[0m: 3 (33%)"),
        ];

        assert_eq!(c.messages(&2), expected);
    }

    #[test]
    fn can_get_top_3() {
        let mut c: Counter = Counter::new();
        c.increment(A);
        c.increment(A);
        c.increment(A);
        c.increment(B);
        c.increment(B);
        c.increment(B);
        c.increment(C);
        c.increment(C);
        c.increment(D);

        let expected = vec![
            String::from("    a\u{1b}[0m: 3 (33%)"),
            String::from("    b\u{1b}[0m: 3 (33%)"),
            String::from("    c\u{1b}[0m: 2 (22%)"),
        ];

        assert_eq!(c.messages(&3), expected);
    }

    #[test]
    fn can_get_top_4() {
        let mut c: Counter = Counter::new();
        c.increment(A);
        c.increment(A);
        c.increment(A);
        c.increment(B);
        c.increment(B);
        c.increment(B);
        c.increment(C);
        c.increment(C);
        c.increment(D);

        let expected = vec![
            String::from("    a\u{1b}[0m: 3 (33%)"),
            String::from("    b\u{1b}[0m: 3 (33%)"),
            String::from("    c\u{1b}[0m: 2 (22%)"),
            String::from("    d\u{1b}[0m: 1 (11%)"),
        ];

        assert_eq!(c.messages(&4), expected);
    }
}
