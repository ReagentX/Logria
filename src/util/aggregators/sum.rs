use crate::util::{
    aggregators::aggregator::{extact_number, Aggregator},
    error::LogriaError,
};

pub struct Sum {
    total: f64,
}

impl Aggregator for Sum {
    fn update(&mut self, message: &str) -> Result<(), LogriaError> {
        if self.total >= f64::MAX {
            self.total = f64::MAX;
        } else if let Some(number) = self.parse(message) {
            self.total += number;
        };
        Ok(())
    }

    fn messages(&self, _: usize) -> Vec<String> {
        vec![format!("    Total: {}", self.total)]
    }
}

impl Sum {
    pub fn new() -> Self {
        Sum { total: 0. }
    }

    fn parse(&self, message: &str) -> Option<f64> {
        extact_number(message)
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

        assert_eq!(sum.messages(1), vec!["    Total: 6"]);
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
