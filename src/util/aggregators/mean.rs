use num_traits::{one, CheckedAdd, CheckedDiv, One};

use crate::util::aggregators::aggregator::Aggregator;
struct Mean<T: Copy + Default + CheckedAdd + CheckedDiv + One> {
    count: T,
    total: T,
}

impl<T: Copy + Default + CheckedAdd + CheckedDiv + One> Aggregator<T> for Mean<T> {
    fn new() -> Mean<T> {
        Mean {
            count: T::default(),
            total: T::default(),
        }
    }

    fn update(&mut self, message: T) {
        self.count = self.count.checked_add(&one::<T>()).unwrap_or_default();
        self.total = self.total.checked_add(&message).unwrap_or_default();
    }

    fn messages(&self, _: usize) -> Vec<String> {
        vec![]
    }
}

impl<T: Copy + Default + CheckedAdd + CheckedDiv + One> Mean<T> {
    fn mean(&self) -> T {
        self.total.checked_div(&self.count).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use crate::util::aggregators::{aggregator::Aggregator, mean::Mean};

    #[test]
    fn mean_int() {
        let mut mean: Mean<i32> = Mean::new();
        mean.update(1);
        mean.update(2);
        mean.update(3);

        assert_eq!(mean.mean(), 2);
        assert_eq!(mean.total, 6);
        assert_eq!(mean.count, 3);
    }

    #[test]
    fn mean_float() {
        let mut mean: Mean<i64> = Mean::new();
        mean.update(1_i64);
        mean.update(2_i64);
        mean.update(3_i64);

        assert!((mean.mean() - 2_i64).abs() == 0);
        assert!((mean.total - 6_i64).abs() == 0);
        assert!((mean.count - 3_i64).abs() == 0);
    }

    #[test]
    fn empty_mean_int() {
        let mean: Mean<i8> = Mean::new();

        assert_eq!(mean.mean(), 0);
        assert_eq!(mean.total, 0);
        assert_eq!(mean.count, 0);
    }

    #[test]
    fn empty_mean_float() {
        let mean: Mean<i32> = Mean::new();

        assert!(mean.mean() == 0);
        assert!(mean.total == 0);
        assert!(mean.count == 0);
    }

    #[test]
    fn mean_int_overflow() {
        let mut mean: Mean<i32> = Mean::new();
        mean.update(i32::MAX - 1);
        mean.update(i32::MAX - 1);

        assert_eq!(mean.mean(), i32::MAX);
        assert_eq!(mean.total, i32::MAX);
        assert_eq!(mean.count, i32::MAX);
    }

    #[test]
    fn mean_float_overflow() {
        let mut mean: Mean<i64> = Mean::new();
        mean.update(i64::MAX - 1_i64);
        mean.update(i64::MAX - 1_i64);

        assert!((mean.mean() - 2_i64).abs() == 0);
        assert!((mean.total - 6_i64).abs() == 0);
        assert!((mean.count - 3_i64).abs() == 0);
    }
}
