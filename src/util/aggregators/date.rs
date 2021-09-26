use std::{
    cmp::{max, min},
    env::consts::DLL_SUFFIX,
    fmt::Display,
    time::Duration,
};

use crate::util::aggregators::aggregator::{AggregationMethod, Aggregator};
use time::{format_description::parse, Date as DateTime};

struct Date {
    format: String,
    earliest: DateTime,
    latest: DateTime,
    count: i64,
    rate: f64,
    unit: String,
}

impl Aggregator<String> for Date {
    fn new(method: &AggregationMethod) -> Self {
        if let AggregationMethod::Date(format_string) = method {
            let parser: Date = Date {
                format: format_string.to_owned(),
                earliest: DateTime::MAX,
                latest: DateTime::MIN,
                count: 0,
                rate: 0.,
                unit: String::from(""),
            };
            parser
        } else {
            panic!("Date aggregator constructed with non-date AggregationMethod!")
        }
    }

    fn update(&mut self, message: String) {
        match parse(&self.format) {
            Ok(parser) => match DateTime::parse(&message, &parser) {
                Ok(date) => {
                    self.earliest = min(date, self.earliest);
                    self.latest = max(date, self.latest);
                    self.count += 1;
                    let rate_data = self.determine_rate();
                    self.rate = rate_data.0;
                    self.unit = rate_data.1;
                }
                Err(why) => {
                    panic!("{}", why.to_string())
                }
            },
            Err(why) => {}
        }
    }

    fn messages(&self, _: usize) -> Vec<String> {
        vec![
            format!("Rate: {:.4} {}", self.rate, self.unit),
            format!("Count: {}", self.count),
            format!("Earliest: {}", self.earliest),
            format!("Latest: {}", self.latest),
        ]
    }
}

impl Date {
    /// Determine the rate at which messages are received
    fn determine_rate(&self) -> (f64, String) {
        let difference = self.latest - self.earliest;
        let mut denominator = difference.whole_weeks();
        let mut unit = "week";
        if difference.whole_days() < self.count {
            denominator = difference.whole_days();
            unit = "day"
        }
        if difference.whole_hours() < self.count {
            denominator = difference.whole_hours();
            unit = "hour"
        }
        if difference.whole_minutes() < self.count {
            denominator = difference.whole_minutes();
            unit = "minute"
        }
        if difference.whole_seconds() < self.count {
            denominator = difference.whole_seconds();
            unit = "second"
        }
        let mut per_unit = String::from("per ");
        per_unit.push_str(unit);
        (self.count as f64 / denominator as f64, per_unit)
    }
}

#[cfg(test)]
mod use_tests {
    use crate::util::aggregators::{
        aggregator::{AggregationMethod, Aggregator},
        date::Date,
    };
    use time::Date as DateTime;

    #[test]
    fn can_construct() {
        let d: Date = Date::new(&AggregationMethod::Date("[month]/[day]/[year]".to_string()));
    }

    #[test]
    fn can_update() {
        let mut d: Date = Date::new(&AggregationMethod::Date("[month]/[day]/[year]".to_string()));
        d.update("01/01/2021".to_string());
        d.update("01/02/2021".to_string());
        d.update("01/03/2021".to_string());
        d.update("01/04/2021".to_string());

        let expected = Date {
            format: "[month]/[day]/[year]".to_string(),
            earliest: DateTime::from_ordinal_date(2021, 1).unwrap(),
            latest: DateTime::from_ordinal_date(2021, 4).unwrap(),
            count: 4,
            rate: 1.3333333333333333,
            unit: String::from("per day"),
        };

        assert_eq!(d.format, expected.format);
        assert_eq!(d.earliest, expected.earliest);
        assert_eq!(d.latest, expected.latest);
        assert_eq!(d.count, expected.count);
        assert_eq!(d.unit, expected.unit);
        assert!(d.rate - expected.rate == 0.);
    }
}

#[cfg(test)]
mod rate_tests {
    use crate::util::aggregators::{
        aggregator::{AggregationMethod, Aggregator},
        date::Date,
    };
    use time::Date as DateTime;

    #[test]
    fn weekly() {
        let d = Date {
            format: "".to_string(),
            earliest: DateTime::from_ordinal_date(2021, 1).unwrap(),
            latest: DateTime::from_ordinal_date(2021, 15).unwrap(),
            count: 3,
            rate: 0.,
            unit: String::from(""),
        };
        assert_eq!(d.determine_rate(), (1.5, "per week".to_string()))
    }

    #[test]
    fn daily() {
        let d = Date {
            format: "".to_string(),
            earliest: DateTime::from_ordinal_date(2021, 1).unwrap(),
            latest: DateTime::from_ordinal_date(2021, 15).unwrap(),
            count: 15,
            rate: 0.,
            unit: String::from(""),
        };
        assert_eq!(
            d.determine_rate(),
            (1.0714285714285714, "per day".to_string())
        )
    }

    #[test]
    fn hourly() {
        let d = Date {
            format: "".to_string(),
            earliest: DateTime::from_ordinal_date(2021, 1).unwrap(),
            latest: DateTime::from_ordinal_date(2021, 3).unwrap(),
            count: 150,
            rate: 0.,
            unit: String::from(""),
        };
        assert_eq!(d.determine_rate(), (3.125, "per hour".to_string()))
    }

    #[test]
    fn minutely() {
        let d = Date {
            format: "".to_string(),
            earliest: DateTime::from_ordinal_date(2021, 1).unwrap(),
            latest: DateTime::from_ordinal_date(2021, 2).unwrap(),
            count: 1500,
            rate: 0.,
            unit: String::from(""),
        };
        assert_eq!(
            d.determine_rate(),
            (1.0416666666666667, "per minute".to_string())
        )
    }

    #[test]
    fn secondly() {
        let d = Date {
            format: "".to_string(),
            earliest: DateTime::from_ordinal_date(2021, 1).unwrap(),
            latest: DateTime::from_ordinal_date(2021, 2).unwrap(),
            count: 100000,
            rate: 0.,
            unit: String::from(""),
        };
        assert_eq!(
            d.determine_rate(),
            (1.1574074074074074, "per second".to_string())
        )
    }
}
