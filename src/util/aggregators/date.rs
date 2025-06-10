use std::cmp::{max, min};

use time::{
    Date as Dt, PrimitiveDateTime as DateTime, Time as Tm,
    format_description::{OwnedFormatItem, parse_owned},
};

use crate::util::{
    aggregators::aggregator::{Aggregator, format_int},
    error::LogriaError,
};

#[derive(Clone, Debug)]
pub enum DateParserType {
    Date,
    Time,
    DateTime,
}

/// Aggregator for tracking temporal data: records earliest and latest timestamps,
/// counts entries, and computes rate over a chosen unit (day, hour, etc.).
pub struct Date {
    /// Parsed format items for the date/time format.
    format: Option<OwnedFormatItem>,
    /// The minimum timestamp observed.
    earliest: DateTime,
    /// The maximum timestamp observed.
    latest: DateTime,
    /// Number of parsed timestamps.
    count: i64,
    /// Specifies whether to parse as Date, Time, or DateTime.
    parser_type: DateParserType,
}

impl Aggregator for Date {
    /// Parses and ingests a new timestamp from `message`, updating internal state.
    fn update(&mut self, message: &str) -> Result<(), LogriaError> {
        match &self.format {
            Some(format) => match self.parser_type {
                DateParserType::Date => match Dt::parse(message, format) {
                    Ok(date) => {
                        self.upsert(DateTime::new(date, Tm::MIDNIGHT));
                        Ok(())
                    }
                    Err(why) => Err(LogriaError::CannotParseDate(why.to_string())),
                },
                DateParserType::Time => match Tm::parse(message, format) {
                    Ok(time) => {
                        self.upsert(DateTime::new(Dt::MIN, time));
                        Ok(())
                    }
                    Err(why) => Err(LogriaError::CannotParseDate(why.to_string())),
                },
                DateParserType::DateTime => match DateTime::parse(message, format) {
                    Ok(date) => {
                        self.upsert(date);
                        Ok(())
                    }
                    Err(why) => Err(LogriaError::CannotParseDate(why.to_string())),
                },
            },
            None => Err(LogriaError::CannotParseDate(
                "No date format string specified!".to_string(),
            )),
        }
    }

    /// Returns vector of formatted output lines: rate, count, earliest, and latest.
    fn messages(&self, _: &usize) -> Vec<String> {
        let (rate, unit) = self.determine_rate();
        let mut out_v = vec![
            format!("    Rate: {} {}", format_int(rate as usize), unit),
            format!("    Count: {}", format_int(self.count as usize)),
        ];
        match self.parser_type {
            DateParserType::Date => {
                out_v.push(format!("    Earliest: {}", self.earliest.date()));
                out_v.push(format!("    Latest: {}", self.latest.date()));
            }
            DateParserType::Time => {
                out_v.push(format!("    Earliest: {}", self.earliest.time()));
                out_v.push(format!("    Latest: {}", self.latest.time()));
            }
            DateParserType::DateTime => {
                out_v.push(format!("    Earliest: {}", self.earliest));
                out_v.push(format!("    Latest: {}", self.latest));
            }
        }
        out_v
    }

    /// Resets the aggregator to its initial state, preserving `format` and `parser_type`.
    fn reset(&mut self) {
        self.count = 0;
        self.earliest = match self.parser_type {
            DateParserType::Date => DateTime::new(Dt::MAX, Tm::MIDNIGHT),
            DateParserType::Time => DateTime::new(Dt::MIN, Tm::from_hms(23, 59, 59).unwrap()),
            DateParserType::DateTime => DateTime::new(Dt::MAX, Tm::MIDNIGHT),
        };
        self.latest = match self.parser_type {
            DateParserType::Date => DateTime::new(Dt::MIN, Tm::MIDNIGHT),
            DateParserType::Time => DateTime::new(Dt::MIN, Tm::MIDNIGHT),
            DateParserType::DateTime => DateTime::new(Dt::MIN, Tm::MIDNIGHT),
        };
    }
}

impl Date {
    /// Constructs a new `Date` aggregator with the given `format` and `parser_type`.
    pub fn new(format: &str, parser_type: DateParserType) -> Self {
        let (earliest, latest) = match parser_type {
            DateParserType::Date => (
                DateTime::new(Dt::MAX, Tm::MIDNIGHT),
                DateTime::new(Dt::MIN, Tm::MIDNIGHT),
            ),
            DateParserType::Time => (
                DateTime::new(Dt::MIN, Tm::from_hms(23, 59, 59).unwrap()),
                DateTime::new(Dt::MIN, Tm::MIDNIGHT),
            ),
            DateParserType::DateTime => (
                DateTime::new(Dt::MAX, Tm::MIDNIGHT),
                DateTime::new(Dt::MIN, Tm::MIDNIGHT),
            ),
        };

        Self {
            format: parse_owned::<2>(format).ok(),
            earliest,
            latest,
            count: 0,
            parser_type,
        }
    }

    /// Inserts `new_date` into the aggregator, adjusting earliest/latest and recalculating rate.
    fn upsert(&mut self, new_date: DateTime) {
        self.earliest = min(new_date, self.earliest);
        self.latest = max(new_date, self.latest);
        self.count += 1;
    }

    /// Calculates the entry rate based on the span between earliest and latest timestamps.
    fn determine_rate(&self) -> (i64, String) {
        let difference = self.latest - self.earliest;
        let mut denominator = difference.whole_weeks();
        let mut unit = "week";
        if difference.whole_days() < self.count {
            denominator = difference.whole_days();
            unit = "day";
        }
        if difference.whole_hours() < self.count {
            denominator = difference.whole_hours();
            unit = "hour";
        }
        if difference.whole_minutes() < self.count {
            denominator = difference.whole_minutes();
            unit = "minute";
        }
        if difference.whole_seconds() < self.count {
            denominator = difference.whole_seconds();
            unit = "second";
        }
        let mut per_unit = String::from("per ");
        per_unit.push_str(unit);
        (
            max(self.count.checked_div(denominator).unwrap_or(self.count), 1),
            per_unit,
        )
    }
}

#[cfg(test)]
mod use_tests {
    use crate::util::aggregators::{
        aggregator::Aggregator,
        date::{Date, DateParserType},
    };
    use time::{
        Date as Dt, PrimitiveDateTime as DateTime, Time as Tm, format_description::parse_owned,
    };

    #[test]
    fn can_construct() {
        Date::new("[month]/[day]/[year]", DateParserType::Date);
        Date::new("[hour]:[minute]:[second]", DateParserType::Time);
        Date::new(
            "[month]/[day]/[year] [hour]:[minute]:[second]",
            DateParserType::DateTime,
        );
    }

    #[test]
    fn can_update_date() {
        let mut d: Date = Date::new("[month]/[day]/[year]", DateParserType::Date);
        d.update("01/01/2021").unwrap();
        d.update("01/02/2021").unwrap();
        d.update("01/03/2021").unwrap();
        d.update("01/04/2021").unwrap();

        let expected = Date {
            earliest: DateTime::new(Dt::from_ordinal_date(2021, 1).unwrap(), Tm::MIDNIGHT),
            latest: DateTime::new(Dt::from_ordinal_date(2021, 4).unwrap(), Tm::MIDNIGHT),
            count: 4,
            parser_type: DateParserType::Date,
            format: parse_owned::<2>("[month]/[day]/[year]").ok(),
        };

        assert_eq!(d.format, expected.format);
        assert_eq!(d.earliest, expected.earliest);
        assert_eq!(d.latest, expected.latest);
        assert_eq!(d.count, expected.count);
        let (rate, unit) = d.determine_rate();
        assert_eq!(unit, "per day");
        assert_eq!(rate, 1);
    }

    #[test]
    fn can_update_time() {
        let mut d: Date = Date::new("[hour]:[minute]:[second]", DateParserType::Time);
        d.update("01:01:00").unwrap();
        d.update("02:01:00").unwrap();
        d.update("03:01:00").unwrap();
        d.update("04:01:00").unwrap();

        let expected = Date {
            earliest: DateTime::new(Dt::MIN, Tm::from_hms(1, 1, 0).unwrap()),
            latest: DateTime::new(Dt::MIN, Tm::from_hms(4, 1, 0).unwrap()),
            count: 4,
            parser_type: DateParserType::Time,
            format: parse_owned::<2>("[hour]:[minute]:[second]").ok(),
        };

        assert_eq!(d.format, expected.format);
        assert_eq!(d.earliest, expected.earliest);
        assert_eq!(d.latest, expected.latest);
        assert_eq!(d.count, expected.count);

        let (rate, unit) = d.determine_rate();
        assert_eq!(unit, "per hour");
        assert_eq!(rate, 1);
    }

    #[test]
    fn can_update_date_time() {
        let mut d: Date = Date::new(
            "[month]/[day]/[year] [hour]:[minute]:[second]",
            DateParserType::DateTime,
        );

        d.update("01/01/2021 01:01:00").unwrap();
        d.update("01/02/2021 02:01:00").unwrap();
        d.update("01/03/2021 03:01:00").unwrap();
        d.update("01/04/2021 04:01:00").unwrap();

        let expected = Date {
            earliest: DateTime::new(
                Dt::from_ordinal_date(2021, 1).unwrap(),
                Tm::from_hms(1, 1, 0).unwrap(),
            ),
            latest: DateTime::new(
                Dt::from_ordinal_date(2021, 4).unwrap(),
                Tm::from_hms(4, 1, 0).unwrap(),
            ),
            count: 4,
            parser_type: DateParserType::DateTime,
            format: parse_owned::<2>("[month]/[day]/[year] [hour]:[minute]:[second]").ok(),
        };

        assert_eq!(d.format, expected.format);
        assert_eq!(d.earliest, expected.earliest);
        assert_eq!(d.latest, expected.latest);
        assert_eq!(d.count, expected.count);

        let (rate, unit) = d.determine_rate();
        assert_eq!(unit, "per day");
        assert_eq!(rate, 1);
    }
}

#[cfg(test)]
mod message_tests {
    use crate::util::aggregators::{
        aggregator::Aggregator,
        date::{Date, DateParserType},
    };

    #[test]
    fn can_update_date() {
        let mut d: Date = Date::new("[month]/[day]/[year]", DateParserType::Date);
        d.update("01/01/2021").unwrap();
        d.update("01/02/2021").unwrap();
        d.update("01/03/2021").unwrap();
        d.update("01/04/2021").unwrap();

        let expected = vec![
            "    Rate: 1 per day".to_string(),
            "    Count: 4".to_string(),
            "    Earliest: 2021-01-01".to_string(),
            "    Latest: 2021-01-04".to_string(),
        ];
        let messages = d.messages(&1);

        assert_eq!(messages, expected);
    }

    #[test]
    fn can_update_time() {
        let mut d: Date = Date::new("[hour]:[minute]:[second]", DateParserType::Time);
        d.update("01:01:00").unwrap();
        d.update("02:01:00").unwrap();
        d.update("03:01:00").unwrap();
        d.update("04:01:00").unwrap();

        let expected = vec![
            "    Rate: 1 per hour".to_string(),
            "    Count: 4".to_string(),
            "    Earliest: 1:01:00.0".to_string(),
            "    Latest: 4:01:00.0".to_string(),
        ];
        let messages = d.messages(&1);

        assert_eq!(messages, expected);
    }

    #[test]
    fn can_update_date_time() {
        let mut d: Date = Date::new(
            "[month]/[day]/[year] [hour]:[minute]:[second]",
            DateParserType::DateTime,
        );
        d.update("01/01/2021 01:01:00").unwrap();
        d.update("01/02/2021 02:01:00").unwrap();
        d.update("01/03/2021 03:01:00").unwrap();
        d.update("01/04/2021 04:01:00").unwrap();

        let expected = vec![
            "    Rate: 1 per day".to_string(),
            "    Count: 4".to_string(),
            "    Earliest: 2021-01-01 1:01:00.0".to_string(),
            "    Latest: 2021-01-04 4:01:00.0".to_string(),
        ];
        let messages = d.messages(&1);

        assert_eq!(messages, expected);
    }
}

#[cfg(test)]
mod rate_tests {
    use crate::util::aggregators::date::{Date, DateParserType};
    use time::{Date as Dt, PrimitiveDateTime as DateTime, Time as Tm};

    #[test]
    fn weekly() {
        let d = Date {
            earliest: DateTime::new(Dt::from_ordinal_date(2021, 1).unwrap(), Tm::MIDNIGHT),
            latest: DateTime::new(Dt::from_ordinal_date(2021, 15).unwrap(), Tm::MIDNIGHT),
            count: 10,
            parser_type: DateParserType::Date,
            format: None,
        };
        assert_eq!(d.determine_rate(), (5, "per week".to_string()));
    }

    #[test]
    fn daily() {
        let d = Date {
            earliest: DateTime::new(Dt::from_ordinal_date(2021, 1).unwrap(), Tm::MIDNIGHT),
            latest: DateTime::new(Dt::from_ordinal_date(2021, 15).unwrap(), Tm::MIDNIGHT),
            count: 15,
            parser_type: DateParserType::Date,
            format: None,
        };
        assert_eq!(d.determine_rate(), (1, "per day".to_string()));
    }

    #[test]
    fn hourly() {
        let d = Date {
            earliest: DateTime::new(Dt::from_ordinal_date(2021, 1).unwrap(), Tm::MIDNIGHT),
            latest: DateTime::new(Dt::from_ordinal_date(2021, 3).unwrap(), Tm::MIDNIGHT),
            count: 150,
            parser_type: DateParserType::Date,
            format: None,
        };
        assert_eq!(d.determine_rate(), (3, "per hour".to_string()));
    }

    #[test]
    fn minutely() {
        let d = Date {
            earliest: DateTime::new(Dt::from_ordinal_date(2021, 1).unwrap(), Tm::MIDNIGHT),
            latest: DateTime::new(Dt::from_ordinal_date(2021, 2).unwrap(), Tm::MIDNIGHT),
            count: 1500,
            parser_type: DateParserType::Date,
            format: None,
        };
        assert_eq!(d.determine_rate(), (1, "per minute".to_string()));
    }

    #[test]
    fn secondly() {
        let d = Date {
            earliest: DateTime::new(Dt::from_ordinal_date(2021, 1).unwrap(), Tm::MIDNIGHT),
            latest: DateTime::new(Dt::from_ordinal_date(2021, 2).unwrap(), Tm::MIDNIGHT),
            count: 100000,
            parser_type: DateParserType::Date,
            format: None,
        };
        assert_eq!(d.determine_rate(), (1, "per second".to_string()));
    }
}
