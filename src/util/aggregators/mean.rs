use crate::util::{
    aggregators::aggregator::{extact_number, Aggregator},
    error::LogriaError,
};

pub struct Mean {
    count: f64,
    total: f64,
}

/// Float implementation of Mean
impl Aggregator for Mean {
    fn update(&mut self, message: &str) -> Result<(), LogriaError> {
        if self.count >= f64::MAX {
            self.count = f64::MAX;
        } else {
            self.count += 1.;
        };

        if self.total >= f64::MAX {
            self.total = f64::MAX;
        } else {
            match self.parse(message) {
                Some(number) => {
                    self.total += number;
                },
                None => {
                    self.count -= 1.;
                }
            }
        };

        Ok(())
    }

    fn messages(&self, _: &usize) -> Vec<String> {
        vec![
            format!("    Mean: {}", self.mean()),
            format!("    Count: {}", self.count),
            format!("    Total: {}", self.total),
        ]
    }
}

impl Mean {
    pub fn new() -> Mean {
        Mean {
            count: 0.,
            total: 0.,
        }
    }

    fn parse(&self, message: &str) -> Option<f64> {
        extact_number(message)
    }

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
                "    Mean: 2".to_string(),
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
