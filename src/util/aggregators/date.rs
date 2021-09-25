use crate::util::aggregators::aggregator::Aggregator;
use time::{format_description::FormatItem, Date as DateTime};

struct Date<'a> {
    format: FormatItem<'a>,
}

impl<T: ToString> Aggregator<T> for Date<'_> {
    fn new() -> Self {
        todo!()
    }

    fn update(&mut self, message: T) {
        todo!()
    }

    fn messages(&self, n: usize) -> Vec<String> {
        todo!()
    }
}

impl Date<'_> {}

#[cfg(test)]
mod int_tests {
    use crate::util::aggregators::{aggregator::Aggregator, date::Date};
    use time::Date as DateTime;

    #[test]
    fn mean() {}
}
