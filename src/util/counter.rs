use std::collections::HashMap;
use std::hash::Hash;

struct Counter<T> {
    state: HashMap<T, usize>,
}

impl<T: Hash> Counter<T> {
    fn new() -> Counter<T> {
        Counter {
            state: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Counter;

    #[test]
    fn can_construct_counter() {
        let c: Counter<String> = Counter::new();
    }
}
