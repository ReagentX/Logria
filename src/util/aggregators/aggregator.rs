use crate::util::error::LogriaError;
use serde::{Deserialize, Serialize};

pub trait Aggregator<T> {
    fn new(method: &AggregationMethod) -> Self;
    fn update(&mut self, message: T) -> Result<(), LogriaError>;
    fn messages(&self, n: usize) -> Vec<String>;
}

#[derive(Eq, PartialEq, Serialize, Deserialize, Debug)]
pub enum AggregationMethod {
    Mean,
    Mode, // Special case of Count, for most_common(1)
    Sum,
    Count,
    Date(String),
    Time(String),
    DateTime(String),
}
