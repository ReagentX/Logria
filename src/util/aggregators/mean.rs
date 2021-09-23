use std::ops::AddAssign;

use num_traits::{one, zero, Float, PrimInt};

use crate::util::aggregators::aggregator::Aggregator;

struct IntMean<T: PrimInt> {
    count: T,
    total: T,
}

/// Integer implementation of Mean
impl<I: PrimInt> Aggregator<I> for IntMean<I> {
    fn new() -> IntMean<I> {
        IntMean {
            count: zero(),
            total: zero(),
        }
    }

    fn update(&mut self, message: I) {
        self.count = self.count.checked_add(&one()).unwrap_or_else(I::max_value);
        self.total = self
            .total
            .checked_add(&message)
            .unwrap_or_else(I::max_value);
    }

    fn messages(&self, _: usize) -> Vec<String> {
        vec![]
    }
}

impl<I: PrimInt> IntMean<I> {
    fn mean(&self) -> I {
        self.total.checked_div(&self.count).unwrap_or_else(zero)
    }
}

struct FloatMean<F: Float + AddAssign> {
    count: F,
    total: F,
}

/// Float implementation of Mean
impl<F: Float + AddAssign> Aggregator<F> for FloatMean<F> {
    fn new() -> FloatMean<F> {
        FloatMean {
            count: zero::<F>(),
            total: zero::<F>(),
        }
    }

    fn update(&mut self, message: F) {
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
    }

    fn messages(&self, _: usize) -> Vec<String> {
        vec![]
    }
}

impl<F: Float + AddAssign> FloatMean<F> {
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
    use crate::util::aggregators::{aggregator::Aggregator, mean::IntMean};

    #[test]
    fn mean() {
        let mut mean: IntMean<i32> = IntMean::new();
        mean.update(1);
        mean.update(2);
        mean.update(3);

        assert_eq!(mean.mean(), 2);
        assert_eq!(mean.total, 6);
        assert_eq!(mean.count, 3);
    }

    #[test]
    fn empty_mean() {
        let mean: IntMean<i8> = IntMean::new();

        assert_eq!(mean.mean(), 0);
        assert_eq!(mean.total, 0);
        assert_eq!(mean.count, 0);
    }

    #[test]
    fn mean_overflow() {
        let mut mean: IntMean<i32> = IntMean::new();
        mean.update(i32::MAX - 1);
        mean.update(i32::MAX - 1);

        assert_eq!(mean.mean(), i32::MAX / 2);
        assert_eq!(mean.total, i32::MAX);
        assert_eq!(mean.count, 2);
    }
}

#[cfg(test)]
mod float_tests {
    use crate::util::aggregators::{aggregator::Aggregator, mean::FloatMean};

    #[test]
    fn mean() {
        let mut mean: FloatMean<f64> = FloatMean::new();
        mean.update(1_f64);
        mean.update(2_f64);
        mean.update(3_f64);

        assert!((mean.mean() - 2_f64).abs() == 0_f64);
        assert!((mean.total - 6_f64).abs() == 0_f64);
        assert!((mean.count - 3_f64).abs() == 0_f64);
    }

    #[test]
    fn empty_mean() {
        let mean: FloatMean<f32> = FloatMean::new();

        assert!(mean.mean() == 0_f32);
        assert!(mean.total == 0_f32);
        assert!(mean.count == 0_f32);
    }

    #[test]
    fn mean_overflow() {
        let mut mean: FloatMean<f64> = FloatMean::new();
        mean.update(f64::MAX - 1_f64);
        mean.update(f64::MAX - 1_f64);

        assert!((mean.mean() - f64::MAX / 2_f64).abs() == 0_f64);
        assert!((mean.total - f64::MAX).abs() == 0_f64);
        assert!((mean.count - 2_f64).abs() == 0_f64);
    }
}
