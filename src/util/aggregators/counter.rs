use std::{
    cmp::Eq,
    collections::{BTreeSet, HashMap},
    fmt::Display,
    hash::Hash,
};

use crate::util::{
    aggregators::aggregator::{AggregationMethod, Aggregator},
    error::LogriaError,
};

/// Counter struct inspired by Python's stdlib Counter class
struct Counter<T: Hash + Eq + Clone + Display + Ord> {
    state: HashMap<T, u64>,
    order: HashMap<u64, BTreeSet<T>>,
}

impl<T: Hash + Eq + Clone + Display + Ord> Aggregator<T> for Counter<T> {
    fn new(_: &AggregationMethod) -> Counter<T> {
        Counter {
            state: HashMap::new(),
            order: HashMap::new(),
        }
    }

    fn update(&mut self, message: T) -> Result<(), LogriaError> {
        self.increment(message);
        Ok(())
    }

    fn messages(&self, n: usize) -> Vec<String> {
        // Place to store the result
        let mut result = Vec::with_capacity(n);
        if n == 0 {
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
                result.push(format!("{}: {}", item, count));
                total_added += 1;
                if total_added == n {
                    return result;
                }
            }
        }

        result
    }
}

impl<T: Hash + Eq + Clone + Display + Ord> Counter<T> {
    /// Determine the total number of items in the Counter
    fn total(&self) -> u64 {
        self.state.values().into_iter().sum()
    }

    /// Remove an item from the internal order store
    fn purge_from_order(&mut self, item: &T, count: &u64) {
        if let Some(order) = self.order.get_mut(count) {
            // If there was data there, remove the existing item
            if !order.is_empty() {
                order.retain(|i| i != item);
                if order.is_empty() {
                    self.order.remove(count);
                }
            };
        };
    }

    fn purge_from_state(&mut self, item: &T) {
        self.state.remove(item);
    }

    /// Update the internal item order HashMap
    fn update_order(&mut self, item: T, old_count: &u64, new_count: &u64) {
        self.purge_from_order(&item, old_count);
        match self.order.get_mut(new_count) {
            Some(v) => {
                v.insert(item);
            }
            None => {
                let mut set = BTreeSet::new();
                set.insert(item);
                self.order.insert(*new_count, set);
            }
        }
    }

    /// Increment an item into the counter, creating if it does not exist
    fn increment(&mut self, item: T) {
        let old_count = self.state.get(&item).unwrap_or(&0).to_owned();
        let new_count = old_count.checked_add(1);
        match new_count {
            Some(count) => self.state.insert(item.to_owned(), count),
            None => self.state.insert(item.to_owned(), old_count),
        };
        self.update_order(item, &old_count, &new_count.unwrap_or(old_count));
    }

    /// Reduce an item from the counter, removing if it becomes 0
    fn decrement(&mut self, item: T) {
        let old_count = self.state.get(&item).unwrap_or(&0).to_owned();
        let new_count = old_count.checked_sub(1);
        match new_count {
            Some(count) => {
                if count > 0 {
                    self.state.insert(item.to_owned(), count);
                    self.update_order(item, &old_count, &count);
                } else {
                    self.delete(&item);
                }
            }
            None => {
                self.delete(&item);
            }
        };
    }

    /// Remove an item from the counter completely
    fn delete(&mut self, item: &T) {
        let count = self.state.get(item).unwrap().to_owned();
        self.purge_from_order(item, &count);
        self.purge_from_state(item);
    }
}

#[cfg(test)]
mod tests {
    use crate::util::aggregators::{
        aggregator::{AggregationMethod::Count, Aggregator},
        counter::Counter,
    };
    use std::collections::{BTreeSet, HashMap};

    static A: &str = "a";
    static B: &str = "b";
    static C: &str = "c";
    static D: &str = "d";

    #[test]
    fn can_construct_counter() {
        let c: Counter<String> = Counter::new(&Count);
    }

    #[test]
    fn can_count_int() {
        let mut c: Counter<i32> = Counter::new(&Count);
        c.increment(1);
        c.increment(1);
        c.increment(1);
        c.increment(2);
        c.increment(2);

        let mut expected_count = HashMap::new();
        expected_count.insert(1, 3);
        expected_count.insert(2, 2);

        let mut expected_order: HashMap<u64, BTreeSet<i32>> = HashMap::new();
        let mut a = BTreeSet::new();
        let mut b = BTreeSet::new();
        a.insert(1);
        b.insert(2);
        expected_order.insert(3, a);
        expected_order.insert(2, b);

        assert_eq!(c.state, expected_count);
        assert_eq!(c.order, expected_order);
    }

    #[test]
    fn can_count() {
        let mut c: Counter<String> = Counter::new(&Count);
        c.increment(A.to_owned());
        c.increment(A.to_owned());
        c.increment(A.to_owned());
        c.increment(B.to_owned());
        c.increment(B.to_owned());

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
        let mut c: Counter<String> = Counter::new(&Count);
        c.update(A.to_owned()).unwrap();
        c.update(A.to_owned()).unwrap();
        c.update(A.to_owned()).unwrap();
        c.update(B.to_owned()).unwrap();
        c.update(B.to_owned()).unwrap();

        let mut expected = HashMap::new();
        expected.insert(A.to_owned(), 3);
        expected.insert(B.to_owned(), 2);

        assert_eq!(c.total(), 5);
    }

    #[test]
    fn can_decrement() {
        let mut c: Counter<String> = Counter::new(&Count);
        c.increment(A.to_owned());
        c.increment(A.to_owned());
        c.increment(A.to_owned());
        c.increment(B.to_owned());
        c.increment(B.to_owned());
        c.decrement(A.to_owned());

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
        let mut c: Counter<String> = Counter::new(&Count);
        c.increment(A.to_owned());
        c.increment(B.to_owned());
        c.increment(B.to_owned());
        c.decrement(A.to_owned());

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
        let mut c: Counter<String> = Counter::new(&Count);
        c.increment(A.to_owned());
        c.increment(A.to_owned());
        c.increment(A.to_owned());
        c.increment(B.to_owned());
        c.increment(B.to_owned());
        c.delete(&A.to_owned());

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
    fn can_get_top_0() {
        let mut c: Counter<String> = Counter::new(&Count);
        c.increment(A.to_owned());
        c.increment(A.to_owned());
        c.increment(A.to_owned());
        c.increment(B.to_owned());
        c.increment(B.to_owned());
        c.increment(B.to_owned());
        c.increment(C.to_owned());
        c.increment(C.to_owned());
        c.increment(D.to_owned());

        let expected: Vec<String> = vec![];

        assert_eq!(c.messages(0), expected);
    }

    #[test]
    fn can_get_top_1() {
        let mut c: Counter<String> = Counter::new(&Count);
        c.increment(A.to_owned());
        c.increment(A.to_owned());
        c.increment(A.to_owned());
        c.increment(B.to_owned());
        c.increment(B.to_owned());
        c.increment(B.to_owned());
        c.increment(C.to_owned());
        c.increment(C.to_owned());
        c.increment(D.to_owned());

        let expected = vec![String::from("a: 3")];

        assert_eq!(c.messages(1), expected);
    }

    #[test]
    fn can_get_top_2() {
        let mut c: Counter<String> = Counter::new(&Count);
        c.increment(A.to_owned());
        c.increment(A.to_owned());
        c.increment(A.to_owned());
        c.increment(B.to_owned());
        c.increment(B.to_owned());
        c.increment(B.to_owned());
        c.increment(C.to_owned());
        c.increment(C.to_owned());
        c.increment(D.to_owned());

        let expected = vec![String::from("a: 3"), String::from("b: 3")];

        assert_eq!(c.messages(2), expected);
    }

    #[test]
    fn can_get_top_3() {
        let mut c: Counter<String> = Counter::new(&Count);
        c.increment(A.to_owned());
        c.increment(A.to_owned());
        c.increment(A.to_owned());
        c.increment(B.to_owned());
        c.increment(B.to_owned());
        c.increment(B.to_owned());
        c.increment(C.to_owned());
        c.increment(C.to_owned());
        c.increment(D.to_owned());

        let expected = vec![
            String::from("a: 3"),
            String::from("b: 3"),
            String::from("c: 2"),
        ];

        assert_eq!(c.messages(3), expected);
    }

    #[test]
    fn can_get_top_4() {
        let mut c: Counter<String> = Counter::new(&Count);
        c.increment(A.to_owned());
        c.increment(A.to_owned());
        c.increment(A.to_owned());
        c.increment(B.to_owned());
        c.increment(B.to_owned());
        c.increment(B.to_owned());
        c.increment(C.to_owned());
        c.increment(C.to_owned());
        c.increment(D.to_owned());

        let expected = vec![
            String::from("a: 3"),
            String::from("b: 3"),
            String::from("c: 2"),
            String::from("d: 1"),
        ];

        assert_eq!(c.messages(4), expected);
    }
}
