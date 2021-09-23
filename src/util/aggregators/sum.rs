use num_traits::{one, zero, Float, PrimInt};
use std::{fmt::Display, ops::AddAssign};

use crate::util::aggregators::aggregator::Aggregator;

/// Integer sum implementation
struct IntSum<I: AddAssign + Display + PrimInt> {
    total: I,
}

impl<I: AddAssign + Display + PrimInt> Aggregator<I> for IntSum<I> {
    fn new() -> Self {
        IntSum { total: zero() }
    }

    fn update(&mut self, message: I) {
        self.total = self
            .total
            .checked_add(&message)
            .unwrap_or_else(I::max_value);
    }

    fn messages(&self, _: usize) -> Vec<String> {
        vec![format!("Total: {}", self.total)]
    }
}

/// Flaot sum implementation
struct FloatSum<F: AddAssign + Display + Float> {
    total: F,
}

impl<F: AddAssign + Display + Float> Aggregator<F> for FloatSum<F> {
    fn new() -> Self {
        FloatSum { total: zero() }
    }

    fn update(&mut self, message: F) {
        if self.total >= F::max_value() {
            self.total = F::max_value()
        } else {
            self.total += message
        };
    }

    fn messages(&self, _: usize) -> Vec<String> {
        vec![format!("Total: {}", self.total)]
    }
}

#[cfg(test)]
mod int_tests {
    use crate::util::aggregators::{aggregator::Aggregator, sum::IntSum};

    #[test]
    fn sum() {
        let mut sum: IntSum<i32> = IntSum::new();
        sum.update(1);
        sum.update(2);
        sum.update(3);

        assert_eq!(sum.total, 6);
    }

    #[test]
    fn sum_empty() {
        let mean: IntSum<u64> = IntSum::new();

        assert_eq!(mean.total, 0);
    }

    #[test]
    fn sum_overflow() {
        let mut sum: IntSum<i8> = IntSum::new();
        sum.update(100);
        sum.update(100);
        sum.update(100);

        assert_eq!(sum.total, i8::MAX);
    }
}

#[cfg(test)]
mod float_tests {
    use crate::util::aggregators::{aggregator::Aggregator, sum::FloatSum};

    #[test]
    fn sum() {
        let mut sum: FloatSum<f32> = FloatSum::new();
        sum.update(1_f32);
        sum.update(2_f32);
        sum.update(3_f32);

        assert!(sum.total - 6_f32 == 0_f32);
    }

    #[test]
    fn sum_empty() {
        let mean: FloatSum<f64> = FloatSum::new();

        assert!(mean.total - 0_f64 == 0_f64);
    }

    #[test]
    fn sum_overflow() {
        let mut sum: FloatSum<f64> = FloatSum::new();
        sum.update(f64::MAX);
        sum.update(f64::MAX);

        assert!(sum.total - f64::MAX == 0_f64);
    }
}
