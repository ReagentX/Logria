use std::collections::{BTreeSet, HashMap};

use crate::{
    constants::cli::colors::RESET_COLOR,
    util::{aggregators::aggregator::Aggregator, error::LogriaError},
};
use format_num::format_num;

/// Counter struct inspired by Python's stdlib Counter class
pub struct Counter {
    state: HashMap<String, u64>,
    order: HashMap<u64, BTreeSet<String>>,
    num_to_get: Option<usize>,
}

impl Aggregator for Counter {
    fn update(&mut self, message: &str) -> Result<(), LogriaError> {
        self.increment(message);
        Ok(())
    }

    fn messages(&self, n: &usize) -> Vec<String> {
        // Place to store the result
        let num = &self.num_to_get.unwrap_or(*n);
        let mut result = Vec::with_capacity(*num);
        if *num == 0_usize {
            return result;
        }

        // Keep track of how many items we have added
        let mut total_added = 0;

        // Get the keys sorted from highest to lowest
        let mut counts: Vec<u64> = self
            .order
            .keys()
            .into_iter()
            .map(|f| f.to_owned())
            .collect();
        counts.sort_unstable();

        // Get the value under each key
        for count in counts.iter().rev() {
            let items = self.order.get(count).unwrap();
            for item in items {
                let total = self.total() as f64;
                result.push(format!(
                    "    {}{}: {} ({:.0}%)",
                    item.trim(),
                    RESET_COLOR,
                    format_num!(",d", *count as f64),
                    (*count as f64 / total) * 100_f64
                ));
                total_added += 1;
                if total_added == *num {
                    return result;
                }
            }
        }
        result
    }
}

impl Counter {
    pub fn new(num_to_get: Option<usize>) -> Counter {
        Counter {
            state: HashMap::new(),
            order: HashMap::new(),
            num_to_get,
        }
    }

    /// Determine the total number of items in the Counter
    fn total(&self) -> u64 {
        self.state.values().into_iter().sum()
    }

    /// Remove an item from the internal order
    fn purge_from_order(&mut self, item: &str, count: &u64) {
        if let Some(order) = self.order.get_mut(count) {
            // If there was data there, remove the existing item
            if !order.is_empty() {
                order.remove(item);
                if order.is_empty() {
                    self.order.remove(count);
                }
            };
        };
    }

    /// Remove an item from the internal state
    fn purge_from_state(&mut self, item: &str) {
        self.state.remove(item);
    }

    /// Update the internal item order HashMap
    fn update_order(&mut self, item: &str, old_count: &u64, new_count: &u64) {
        self.purge_from_order(item, old_count);
        match self.order.get_mut(new_count) {
            Some(v) => {
                v.insert(item.to_owned());
            }
            None => {
                let mut set = BTreeSet::new();
                set.insert(item.to_owned());
                self.order.insert(*new_count, set);
            }
        }
    }

    /// Increment an item into the counter, creating if it does not exist
    fn increment(&mut self, item: &str) {
        let old_count = self.state.get(item).unwrap_or(&0).to_owned();
        let new_count = old_count.checked_add(1).unwrap_or(old_count);
        self.state.insert(item.to_owned(), new_count);
        self.update_order(item, &old_count, &new_count);
    }

    /// Reduce an item from the counter, removing if it becomes 0
    fn decrement(&mut self, item: &str) {
        let old_count = self.state.get(item).unwrap_or(&0).to_owned();
        let new_count = old_count.checked_sub(1);
        match new_count {
            Some(count) => {
                if count > 0 {
                    self.state.insert(item.to_owned(), count);
                    self.update_order(item, &old_count, &count);
                } else {
                    self.delete(item);
                }
            }
            None => {
                self.delete(item);
            }
        };
    }

    /// Remove an item from the counter completely
    fn delete(&mut self, item: &str) {
        let count = self.state.get(item).unwrap().to_owned();
        self.purge_from_order(item, &count);
        self.purge_from_state(item);
    }
}

#[cfg(test)]
mod behavior_tests {
    use crate::util::aggregators::{aggregator::Aggregator, counter::Counter};
    use std::collections::{BTreeSet, HashMap};

    static A: &str = "a";
    static B: &str = "b";

    #[test]
    fn can_construct_counter() {
        Counter::new(None);
    }

    #[test]
    fn can_count_int() {
        let mut c: Counter = Counter::new(None);
        c.increment("1");
        c.increment("1");
        c.increment("1");
        c.increment("2");
        c.increment("2");

        let mut expected_count = HashMap::new();
        expected_count.insert("1".to_string(), 3);
        expected_count.insert("2".to_string(), 2);

        let mut expected_order: HashMap<u64, BTreeSet<String>> = HashMap::new();
        let mut a = BTreeSet::new();
        let mut b = BTreeSet::new();
        a.insert("1".to_string());
        b.insert("2".to_string());
        expected_order.insert(3, a);
        expected_order.insert(2, b);

        assert_eq!(c.state, expected_count);
        assert_eq!(c.order, expected_order);
    }

    #[test]
    fn can_count() {
        let mut c: Counter = Counter::new(Some(5));
        c.increment(A);
        c.increment(A);
        c.increment(A);
        c.increment(B);
        c.increment(B);

        let mut expected_count = HashMap::new();
        expected_count.insert(A.to_owned(), 3);
        expected_count.insert(B.to_owned(), 2);

        let mut expected_order: HashMap<u64, BTreeSet<String>> = HashMap::new();
        let mut a = BTreeSet::new();
        let mut b = BTreeSet::new();
        a.insert(A.to_owned());
        b.insert(B.to_owned());
        expected_order.insert(3, a);
        expected_order.insert(2, b);

        assert_eq!(c.state, expected_count);
        assert_eq!(c.order, expected_order);
    }

    #[test]
    fn can_sum() {
        let mut c: Counter = Counter::new(None);
        c.update(A).unwrap();
        c.update(A).unwrap();
        c.update(A).unwrap();
        c.update(B).unwrap();
        c.update(B).unwrap();

        let mut expected = HashMap::new();
        expected.insert(A.to_owned(), 3);
        expected.insert(B.to_owned(), 2);

        assert_eq!(c.total(), 5);
    }

    #[test]
    fn can_decrement() {
        let mut c: Counter = Counter::new(Some(5));
        c.increment(A);
        c.increment(A);
        c.increment(A);
        c.increment(B);
        c.increment(B);
        c.decrement(A);

        let mut expected_count = HashMap::new();
        expected_count.insert(A.to_owned(), 2);
        expected_count.insert(B.to_owned(), 2);

        let mut expected_order: HashMap<u64, BTreeSet<String>> = HashMap::new();
        let mut a = BTreeSet::new();
        a.insert(A.to_owned());
        a.insert(B.to_owned());
        expected_order.insert(2, a);

        assert_eq!(c.state, expected_count);
        assert_eq!(c.order, expected_order);
    }

    #[test]
    fn can_decrement_auto_remove() {
        let mut c: Counter = Counter::new(Some(5));
        c.increment(A);
        c.increment(B);
        c.increment(B);
        c.decrement(A);

        let mut expected_count = HashMap::new();
        expected_count.insert(B.to_owned(), 2);

        let mut expected_order: HashMap<u64, BTreeSet<String>> = HashMap::new();
        let mut b = BTreeSet::new();
        b.insert(B.to_owned());
        expected_order.insert(2, b);

        assert_eq!(c.state, expected_count);
        assert_eq!(c.order, expected_order);
    }

    #[test]
    fn can_delete() {
        let mut c: Counter = Counter::new(Some(5));
        c.increment(A);
        c.increment(A);
        c.increment(A);
        c.increment(B);
        c.increment(B);
        c.delete(A);

        let mut expected_count = HashMap::new();
        expected_count.insert(B.to_owned(), 2);

        let mut expected_order: HashMap<u64, BTreeSet<String>> = HashMap::new();
        let mut b = BTreeSet::new();
        b.insert(B.to_owned());
        expected_order.insert(2, b);

        assert_eq!(c.state, expected_count);
        assert_eq!(c.order, expected_order);
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
        let mut c: Counter = Counter::new(None);
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
        let mut c: Counter = Counter::new(None);
        c.increment(A);
        c.increment(A);
        c.increment(A);
        c.increment(B);
        c.increment(B);
        c.increment(B);
        c.increment(C);
        c.increment(C);
        c.increment(D);

        let expected = vec![String::from("    a\u{1b}[0m: 3 (33%)")];

        assert_eq!(c.messages(&1), expected);
    }

    #[test]
    fn can_get_top_2() {
        let mut c: Counter = Counter::new(None);
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
        let mut c: Counter = Counter::new(None);
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
        let mut c: Counter = Counter::new(Some(5));
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
