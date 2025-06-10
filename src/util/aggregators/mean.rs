use crate::util::{
    aggregators::aggregator::{Aggregator, extract_number, format_float},
    error::LogriaError,
};

/// Aggregator that computes the running mean of numeric messages.
pub struct Mean {
    /// Number of numeric messages processed.
    count: f64,
    /// Sum of parsed numeric message values.
    total: f64,
}

/// Float implementation of Mean
impl Aggregator for Mean {
    /// Parses `message`, updates the running count, and accumulates the total.
    /// Saturates `count` and `total` at `f64::MAX` to prevent overflow.
    fn update(&mut self, message: &str) -> Result<(), LogriaError> {
        if self.count >= f64::MAX {
            self.count = f64::MAX;
        } else {
            self.count += 1.;
        }

        if self.total >= f64::MAX {
            self.total = f64::MAX;
        } else {
            match self.parse(message) {
                Some(number) => {
                    self.total += number;
                }
                None => {
                    self.count -= 1.;
                }
            }
        }

        Ok(())
    }

    /// Returns formatted output: current mean (two decimals), count, and total.
    fn messages(&self, _: &usize) -> Vec<String> {
        vec![
            format!("    Mean: {:.2}", self.mean()),
            format!("    Count: {}", format_float(self.count)),
            format!("    Total: {}", format_float(self.total)),
        ]
    }

    /// Resets both `count` and `total` back to zero.
    fn reset(&mut self) {
        self.count = 0.;
        self.total = 0.;
    }
}

impl Mean {
    /// Creates a new `Mean` aggregator with zero count and total.
    pub fn new() -> Mean {
        Mean {
            count: 0.,
            total: 0.,
        }
    }

    /// Attempts to parse a floating-point number from `message`.
    /// Returns `None` if parsing fails.
    fn parse(&self, message: &str) -> Option<f64> {
        extract_number(message)
    }

    /// Computes the current average; returns `total` if no values have been aggregated.
    fn mean(&self) -> f64 {
        if self.count == 0. {
            self.total
        } else {
            self.total / self.count
        }
    }
}

#[cfg(test)]
mod float_tests {
    use crate::util::aggregators::{aggregator::Aggregator, mean::Mean};

    #[test]
    fn mean() {
        let mut mean: Mean = Mean::new();
        mean.update("1_f64").unwrap();
        mean.update("2_f64").unwrap();
        mean.update("3_f64").unwrap();

        assert!((mean.mean() - 2_f64).abs() == 0_f64);
        assert!((mean.total - 6_f64).abs() == 0_f64);
        assert!((mean.count - 3_f64).abs() == 0_f64);
    }

    #[test]
    fn display() {
        let mut mean: Mean = Mean::new();
        mean.update("1_f64").unwrap();
        mean.update("2_f64").unwrap();
        mean.update("3_f64").unwrap();

        assert_eq!(
            mean.messages(&1),
            vec![
                "    Mean: 2.00".to_string(),
                "    Count: 3".to_string(),
                "    Total: 6".to_string(),
            ]
        );
    }

    #[test]
    fn empty_mean() {
        let mean: Mean = Mean::new();

        assert!(mean.mean() == 0_f64);
        assert!(mean.total == 0_f64);
        assert!(mean.count == 0_f64);
    }

    #[test]
    fn mean_overflow() {
        let mut mean: Mean = Mean::new();
        mean.update(&format!("{}test", f64::MAX - 1_f64)).unwrap();
        mean.update(&format!("{} test", f64::MAX - 1_f64)).unwrap();

        assert!((mean.mean() - f64::MAX / 2_f64).abs() == 0_f64);
        assert!((mean.total - f64::MAX).abs() == 0_f64);
        assert!((mean.count - 2_f64).abs() == 0_f64);
    }
}
