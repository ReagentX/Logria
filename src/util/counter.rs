use std::collections::HashMap;
use std::hash::Hash;

/// Counter struct inspired by Python's stdlib Counter class
/// https://github.com/python/cpython/blob/main/Lib/collections/__init__.py
struct Counter<T> {
    state: HashMap<T, usize>,
}

struct Item {
    value: String,
    count: usize,
}

impl<T: Hash> Counter<T> {
    fn new() -> Counter<T> {
        Counter {
            state: HashMap::new(),
        }
    }

    /// Determine the total number of items in the Counter
    fn total(&self) -> usize {
        self.state.values().into_iter().sum()
    }

    /// Get the `n` most common items in the Counter
    fn most_common(&self, n: usize) -> Vec<Item> {
        vec![]
    }

    /// Increment an item into the counter, creating if it does not exist
    fn increment(&self, item: T) {}

    /// Reduce an item from the counter, removing if it becomes 0
    fn decrement(&self, item: T) {}

    /// Remove an item from the counter
    fn delete(&self, item: T) {}
}

#[cfg(test)]
mod tests {
    use super::Counter;

    #[test]
    fn can_construct_counter() {
        let c: Counter<String> = Counter::new();
    }
}
