use serde::{Deserialize, Serialize};

pub trait Aggregator<T> {
    fn new() -> Self;
    fn update(&mut self, message: T);
    fn messages(&self, n: usize) -> Vec<String>;
}

#[derive(Eq, PartialEq, Serialize, Deserialize, Debug)]
pub enum AggregationMethod {
    Mean,
    Median,
    Mode,
    Sum,
    Count,
    Date,
}
