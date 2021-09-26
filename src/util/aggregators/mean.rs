use std::{fmt::Display, ops::AddAssign};

use num_traits::{one, zero, Float, PrimInt};

use crate::util::{
    aggregators::aggregator::{AggregationMethod, Aggregator},
    error::LogriaError,
};

struct IntMean<T: PrimInt + Display> {
    count: T,
    total: T,
}

/// Integer implementation of Mean
impl<I: PrimInt + Display> Aggregator<I> for IntMean<I> {
    fn new(_: &AggregationMethod) -> IntMean<I> {
        IntMean {
            count: zero(),
            total: zero(),
        }
    }

    fn update(&mut self, message: I) -> Result<(), LogriaError> {
        self.count = self.count.checked_add(&one()).unwrap_or_else(I::max_value);
        self.total = self
            .total
            .checked_add(&message)
            .unwrap_or_else(I::max_value);
        Ok(())
    }

    fn messages(&self, _: usize) -> Vec<String> {
        vec![
            format!("    Mean: {}", self.mean()),
            format!("    Count: {}", self.count),
            format!("    Total: {}", self.total),
        ]
    }
}

impl<I: PrimInt + Display> IntMean<I> {
    fn mean(&self) -> I {
        self.total.checked_div(&self.count).unwrap_or_else(zero)
    }
}

struct FloatMean<F: Float + AddAssign + Display> {
    count: F,
    total: F,
}

/// Float implementation of Mean
impl<F: Float + AddAssign + Display> Aggregator<F> for FloatMean<F> {
    fn new(_: &AggregationMethod) -> FloatMean<F> {
        FloatMean {
            count: zero::<F>(),
            total: zero::<F>(),
        }
    }

    fn update(&mut self, message: F) -> Result<(), LogriaError> {
        if self.count >= F::max_value() {
            self.count = F::max_value()
        } else {
            self.count += one::<F>();
        };

        if self.total >= F::max_value() {
            self.total = F::max_value()
        } else {
            self.total += message
        };

        Ok(())
    }

    fn messages(&self, _: usize) -> Vec<String> {
        vec![
            format!("    Mean: {}", self.mean()),
            format!("    Count: {}", self.count),
            format!("    Total: {}", self.total),
        ]
    }
}

impl<F: Float + AddAssign + Display> FloatMean<F> {
    fn mean(&self) -> F {
        if self.count == zero::<F>() {
            self.total
        } else {
            self.total / self.count
        }
    }
}

#[cfg(test)]
mod int_tests {
    use crate::util::aggregators::{
        aggregator::{AggregationMethod::Mean, Aggregator},
        mean::IntMean,
    };

    #[test]
    fn mean() {
        let mut mean: IntMean<i32> = IntMean::new(&Mean);
        mean.update(1).unwrap();
        mean.update(2).unwrap();
        mean.update(3).unwrap();

        assert_eq!(mean.mean(), 2);
        assert_eq!(mean.total, 6);
        assert_eq!(mean.count, 3);
    }

    #[test]
    fn display() {
        let mut mean: IntMean<i32> = IntMean::new(&Mean);
        mean.update(1).unwrap();
        mean.update(2).unwrap();
        mean.update(3).unwrap();

        assert_eq!(
            mean.messages(1),
            vec![
                "    Mean: 2".to_string(),
                "    Count: 3".to_string(),
                "    Total: 6".to_string(),
            ]
        );
    }

    #[test]
    fn empty_mean() {
        let mean: IntMean<i8> = IntMean::new(&Mean);

        assert_eq!(mean.mean(), 0);
        assert_eq!(mean.total, 0);
        assert_eq!(mean.count, 0);
    }

    #[test]
    fn mean_overflow() {
        let mut mean: IntMean<i32> = IntMean::new(&Mean);
        mean.update(i32::MAX - 1).unwrap();
        mean.update(i32::MAX - 1).unwrap();

        assert_eq!(mean.mean(), i32::MAX / 2);
        assert_eq!(mean.total, i32::MAX);
        assert_eq!(mean.count, 2);
    }
}

#[cfg(test)]
mod float_tests {
    use crate::util::aggregators::{
        aggregator::{AggregationMethod::Mean, Aggregator},
        mean::FloatMean,
    };

    #[test]
    fn mean() {
        let mut mean: FloatMean<f64> = FloatMean::new(&Mean);
        mean.update(1_f64).unwrap();
        mean.update(2_f64).unwrap();
        mean.update(3_f64).unwrap();

        assert!((mean.mean() - 2_f64).abs() == 0_f64);
        assert!((mean.total - 6_f64).abs() == 0_f64);
        assert!((mean.count - 3_f64).abs() == 0_f64);
    }

    #[test]
    fn display() {
        let mut mean: FloatMean<f64> = FloatMean::new(&Mean);
        mean.update(1_f64).unwrap();
        mean.update(2_f64).unwrap();
        mean.update(3_f64).unwrap();

        assert_eq!(
            mean.messages(1),
            vec![
                "    Mean: 2".to_string(),
                "    Count: 3".to_string(),
                "    Total: 6".to_string(),
            ]
        );
    }

    #[test]
    fn empty_mean() {
        let mean: FloatMean<f32> = FloatMean::new(&Mean);

        assert!(mean.mean() == 0_f32);
        assert!(mean.total == 0_f32);
        assert!(mean.count == 0_f32);
    }

    #[test]
    fn mean_overflow() {
        let mut mean: FloatMean<f64> = FloatMean::new(&Mean);
        mean.update(f64::MAX - 1_f64).unwrap();
        mean.update(f64::MAX - 1_f64).unwrap();

        assert!((mean.mean() - f64::MAX / 2_f64).abs() == 0_f64);
        assert!((mean.total - f64::MAX).abs() == 0_f64);
        assert!((mean.count - 2_f64).abs() == 0_f64);
    }
}
