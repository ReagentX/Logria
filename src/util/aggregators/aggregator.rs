use serde::{Deserialize, Serialize};

pub trait Aggregator<T> {
    fn new() -> Self;
    fn update(&mut self, message: T);
    fn messages(&self, n: usize) -> Vec<String>;
}

#[derive(Eq, PartialEq, Serialize, Deserialize, Debug)]
pub enum AggregationMethod {
    Mean,         // Done
    Mode,         // Special case of Count, for most_common(1)
    Sum,          // Done
    Count,        // Done
    Date(String), // TODO: Date
}
