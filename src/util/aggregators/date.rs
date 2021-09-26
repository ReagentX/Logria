use std::fmt::Display;

use crate::util::aggregators::aggregator::{AggregationMethod, Aggregator};
use time::{
    format_description::{parse, FormatItem},
    Date as DateTime,
};

struct Date<'a, T: Display> {
    format: String,
    formatter: Vec<FormatItem<'a>>,
    earliest: Option<T>,
    latest: Option<T>,
}

impl<'a, T: Display> Aggregator<'a, T> for Date<'a, T> {
    fn new(method: AggregationMethod) -> Self {
        if let AggregationMethod::Date(format_string) = method {
            match parse(&format_string) {
                Ok(formatter) => {
                    let parser: Date<'a, T> = Date {
                        format: format_string.to_owned(),
                        formatter,
                        earliest: None,
                        latest: None,
                    };
                    return parser;
                }
                Err(why) => panic!(why),
            }
        } else {
            panic!("Date aggregator constructed with non-date AggregationMethod!")
        };
    }

    fn update(&mut self, message: T) {
        todo!()
    }

    fn messages(&self, n: usize) -> Vec<String> {
        todo!()
    }
}

impl<'a, T: Display> Date<'a, T> {}

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
