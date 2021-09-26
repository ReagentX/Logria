use std::fmt::Display;

use crate::util::aggregators::aggregator::{AggregationMethod, Aggregator};
use time::{
    format_description::{parse, FormatItem},
    Date as DateTime,
};

struct Date<T: Display> {
    format: String,
    earliest: Option<T>,
    latest: Option<T>,
}

impl<T: Display> Aggregator<T> for Date<T> {
    fn new(method: &AggregationMethod) -> Self {
        if let AggregationMethod::Date(format_string) = method {
            let parser: Date<T> = Date {
                format: format_string.to_owned(),
                earliest: None,
                latest: None,
            };
            parser
        } else {
            panic!("Date aggregator constructed with non-date AggregationMethod!")
        }
    }

    fn update(&mut self, message: T) {
        todo!()
    }

    fn messages(&self, n: usize) -> Vec<String> {
        todo!()
    }
}

impl<T: Display> Date<T> {}

#[cfg(test)]
mod int_tests {
    use crate::util::aggregators::{
        aggregator::{AggregationMethod, Aggregator},
        date::Date,
    };
    use time::Date as DateTime;

    #[test]
    fn can_construct() {
        let date_ag: Date<String> =
            Date::new(&AggregationMethod::Date("[month]/[day]/[year]".to_string()));
    }
}
