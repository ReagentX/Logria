use crate::util::{
    aggregators::aggregator::{Aggregator, extract_number},
    error::LogriaError,
};
use format_num::format_num;

/// Aggregator that accumulates numeric values from messages into a running total.
pub struct Sum {
    /// Running total of numeric messages.
    total: f64,
}

impl Aggregator for Sum {
    /// Parses `message`, extracts a number if present, and adds it to the total.
    /// Saturates at `f64::MAX` on overflow.
    fn update(&mut self, message: &str) -> Result<(), LogriaError> {
        if self.total >= f64::MAX {
            self.total = f64::MAX;
        } else if let Some(number) = self.parse(message) {
            self.total += number;
        }
        Ok(())
    }

    /// Returns the current total formatted as a string message.
    fn messages(&self, _: &usize) -> Vec<String> {
        vec![format!("    Total: {}", format_num!(",d", self.total))]
    }

    /// Resets the running total back to zero.
    fn reset(&mut self) {
        self.total = 0.;
    }
}

impl Sum {
    /// Creates a new `Sum` aggregator with an initial total of zero.
    pub fn new() -> Self {
        Sum { total: 0. }
    }

    /// Attempts to parse a numeric value from `message`, returning `None` if parsing fails.
    fn parse(&self, message: &str) -> Option<f64> {
        extract_number(message)
    }
}

#[cfg(test)]
mod float_tests {
    use crate::util::aggregators::{aggregator::Aggregator, sum::Sum};

    #[test]
    fn sum() {
        let mut sum: Sum = Sum::new();
        sum.update("1_f32").unwrap();
        sum.update("2_f32").unwrap();
        sum.update("3_f32").unwrap();

        assert!(sum.total - 6. == 0.);
    }

    #[test]
    fn messages() {
        let mut sum: Sum = Sum::new();
        sum.update("1_f32").unwrap();
        sum.update("2_f32").unwrap();
        sum.update("3_f32").unwrap();

        assert_eq!(sum.messages(&1), vec!["    Total: 6"]);
    }

    #[test]
    fn sum_empty() {
        let mean: Sum = Sum::new();

        assert!(mean.total - 0_f64 == 0_f64);
    }

    #[test]
    fn sum_overflow() {
        let mut sum: Sum = Sum::new();
        sum.update(&format!("{}test", f64::MAX)).unwrap();
        sum.update(&format!("{} test", f64::MAX)).unwrap();

        assert!(sum.total - f64::MAX == 0_f64);
    }
}
