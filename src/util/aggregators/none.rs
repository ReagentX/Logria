use crate::util::{aggregators::aggregator::Aggregator, error::LogriaError};

pub struct NoneAg {}

impl Aggregator for NoneAg {
    fn update(&mut self, _: &str) -> Result<(), LogriaError> {
        Ok(())
    }

    fn messages(&self, _: &usize) -> Vec<String> {
        vec!["    Disabled".to_owned()]
    }
}

impl NoneAg {
    pub fn new() -> Self {
        NoneAg {}
    }
}
