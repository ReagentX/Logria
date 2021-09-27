use std::cmp::{max, min};

use crate::util::{aggregators::aggregator::Aggregator, error::LogriaError};
use time::{format_description::parse, Date as Dt, PrimitiveDateTime as DateTime, Time as Tm};

pub enum DateParserType {
    Date,
    Time,
    DateTime,
}

pub struct Date {
    format: String,
    earliest: DateTime,
    latest: DateTime,
    count: i64,
    rate: i64,
    unit: String,
    parser_type: DateParserType,
}

impl Aggregator for Date {
    fn update(&mut self, message: &str) -> Result<(), LogriaError> {
        match parse(&self.format) {
            Ok(parser) => match self.parser_type {
                DateParserType::Date => match Dt::parse(message, &parser) {
                    Ok(date) => {
                        self.upsert(DateTime::new(date, Tm::MIDNIGHT));
                        Ok(())
                    }
                    Err(why) => Err(LogriaError::CannotParseDate(why.to_string())),
                },
                DateParserType::Time => match Tm::parse(message, &parser) {
                    Ok(time) => {
                        self.upsert(DateTime::new(Dt::MIN, time));
                        Ok(())
                    }
                    Err(why) => Err(LogriaError::CannotParseDate(why.to_string())),
                },
                DateParserType::DateTime => match DateTime::parse(message, &parser) {
                    Ok(date) => {
                        self.upsert(date);
                        Ok(())
                    }
                    Err(why) => Err(LogriaError::CannotParseDate(why.to_string())),
                },
            },
            Err(why) => panic!("{}", why),
        }
    }

    fn messages(&self, _: usize) -> Vec<String> {
        let mut out_v = vec![
            format!("    Rate: {:.4} {}", self.rate, self.unit),
            format!("    Count: {}", self.count),
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
        };
        out_v
    }
}

impl Date {
    pub fn new(format: &str, format_type: DateParserType) -> Self {
        match format_type {
            // If we only care about the date, set the time to midnight
            DateParserType::Date => Date {
                format: format.to_owned(),
                earliest: DateTime::new(Dt::MAX, Tm::MIDNIGHT),
                latest: DateTime::new(Dt::MIN, Tm::MIDNIGHT),
                count: 0,
                rate: 0,
                unit: String::from(""),
                parser_type: DateParserType::Date,
            },
            // If we only care about the time, use the same date and the latest/earliset possible times
            DateParserType::Time => Date {
                format: format.to_owned(),
                earliest: DateTime::new(Dt::MIN, Tm::from_hms(23, 59, 59).unwrap()),
                latest: DateTime::new(Dt::MIN, Tm::MIDNIGHT),
                count: 0,
                rate: 0,
                unit: String::from(""),
                parser_type: DateParserType::Time,
            },
            DateParserType::DateTime => Date {
                format: format.to_owned(),
                earliest: DateTime::new(Dt::MAX, Tm::MIDNIGHT),
                latest: DateTime::new(Dt::MIN, Tm::MIDNIGHT),
                count: 0,
                rate: 0,
                unit: String::from(""),
                parser_type: DateParserType::DateTime,
            },
        }
    }

    fn upsert(&mut self, new_date: DateTime) {
        self.earliest = min(new_date, self.earliest);
        self.latest = max(new_date, self.latest);
        self.count += 1;
        let rate_data = self.determine_rate();
        self.rate = rate_data.0;
        self.unit = rate_data.1;
    }

    /// Determine the rate at which messages are received
    fn determine_rate(&self) -> (i64, String) {
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
        (self.count.checked_div(denominator).unwrap_or(0), per_unit)
    }
}

#[cfg(test)]
mod use_tests {
    use crate::util::aggregators::{
        aggregator::Aggregator,
        date::{Date, DateParserType},
    };
    use time::{Date as Dt, PrimitiveDateTime as DateTime, Time as Tm};

    #[test]
    fn can_construct() {
        let d: Date = Date::new("[month]/[day]/[year]", DateParserType::Date);
    }

    #[test]
    fn can_update_date() {
        let mut d: Date = Date::new("[month]/[day]/[year]", DateParserType::Date);
        d.update("01/01/2021").unwrap();
        d.update("01/02/2021").unwrap();
        d.update("01/03/2021").unwrap();
        d.update("01/04/2021").unwrap();

        let expected = Date {
            format: "[month]/[day]/[year]".to_string(),
            earliest: DateTime::new(Dt::from_ordinal_date(2021, 1).unwrap(), Tm::MIDNIGHT),
            latest: DateTime::new(Dt::from_ordinal_date(2021, 4).unwrap(), Tm::MIDNIGHT),
            count: 4,
            rate: 1,
            unit: String::from("per day"),
            parser_type: DateParserType::Date,
        };

        assert_eq!(d.format, expected.format);
        assert_eq!(d.earliest, expected.earliest);
        assert_eq!(d.latest, expected.latest);
        assert_eq!(d.count, expected.count);
        assert_eq!(d.unit, expected.unit);
        assert_eq!(d.rate, expected.rate);
    }

    #[test]
    fn can_update_time() {
        let mut d: Date = Date::new("[hour]:[minute]:[second]", DateParserType::Time);
        d.update("01:01:00").unwrap();
        d.update("02:01:00").unwrap();
        d.update("03:01:00").unwrap();
        d.update("04:01:00").unwrap();

        let expected = Date {
            format: "[hour]:[minute]:[second]".to_string(),
            earliest: DateTime::new(Dt::MIN, Tm::from_hms(1, 1, 0).unwrap()),
            latest: DateTime::new(Dt::MIN, Tm::from_hms(4, 1, 0).unwrap()),
            count: 4,
            rate: 1,
            unit: String::from("per hour"),
            parser_type: DateParserType::Time,
        };

        assert_eq!(d.format, expected.format);
        assert_eq!(d.earliest, expected.earliest);
        assert_eq!(d.latest, expected.latest);
        assert_eq!(d.count, expected.count);
        assert_eq!(d.unit, expected.unit);
        assert_eq!(d.rate, expected.rate);
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
            format: "[month]/[day]/[year] [hour]:[minute]:[second]".to_string(),
            earliest: DateTime::new(
                Dt::from_ordinal_date(2021, 1).unwrap(),
                Tm::from_hms(1, 1, 0).unwrap(),
            ),
            latest: DateTime::new(
                Dt::from_ordinal_date(2021, 4).unwrap(),
                Tm::from_hms(4, 1, 0).unwrap(),
            ),
            count: 4,
            rate: 1,
            unit: String::from("per day"),
            parser_type: DateParserType::DateTime,
        };

        assert_eq!(d.format, expected.format);
        assert_eq!(d.earliest, expected.earliest);
        assert_eq!(d.latest, expected.latest);
        assert_eq!(d.count, expected.count);
        assert_eq!(d.unit, expected.unit);
        assert_eq!(d.rate, expected.rate);
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
        let messages = d.messages(1);

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
        let messages = d.messages(1);

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
        let messages = d.messages(1);

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
            format: "".to_string(),
            earliest: DateTime::new(Dt::from_ordinal_date(2021, 1).unwrap(), Tm::MIDNIGHT),
            latest: DateTime::new(Dt::from_ordinal_date(2021, 15).unwrap(), Tm::MIDNIGHT),
            count: 10,
            rate: 0,
            unit: String::from(""),
            parser_type: DateParserType::Date,
        };
        assert_eq!(d.determine_rate(), (5, "per week".to_string()))
    }

    #[test]
    fn daily() {
        let d = Date {
            format: "".to_string(),
            earliest: DateTime::new(Dt::from_ordinal_date(2021, 1).unwrap(), Tm::MIDNIGHT),
            latest: DateTime::new(Dt::from_ordinal_date(2021, 15).unwrap(), Tm::MIDNIGHT),
            count: 15,
            rate: 0,
            unit: String::from(""),
            parser_type: DateParserType::Date,
        };
        assert_eq!(d.determine_rate(), (1, "per day".to_string()))
    }

    #[test]
    fn hourly() {
        let d = Date {
            format: "".to_string(),
            earliest: DateTime::new(Dt::from_ordinal_date(2021, 1).unwrap(), Tm::MIDNIGHT),
            latest: DateTime::new(Dt::from_ordinal_date(2021, 3).unwrap(), Tm::MIDNIGHT),
            count: 150,
            rate: 0,
            unit: String::from(""),
            parser_type: DateParserType::Date,
        };
        assert_eq!(d.determine_rate(), (3, "per hour".to_string()))
    }

    #[test]
    fn minutely() {
        let d = Date {
            format: "".to_string(),
            earliest: DateTime::new(Dt::from_ordinal_date(2021, 1).unwrap(), Tm::MIDNIGHT),
            latest: DateTime::new(Dt::from_ordinal_date(2021, 2).unwrap(), Tm::MIDNIGHT),
            count: 1500,
            rate: 0,
            unit: String::from(""),
            parser_type: DateParserType::Date,
        };
        assert_eq!(d.determine_rate(), (1, "per minute".to_string()))
    }

    #[test]
    fn secondly() {
        let d = Date {
            format: "".to_string(),
            earliest: DateTime::new(Dt::from_ordinal_date(2021, 1).unwrap(), Tm::MIDNIGHT),
            latest: DateTime::new(Dt::from_ordinal_date(2021, 2).unwrap(), Tm::MIDNIGHT),
            count: 100000,
            rate: 0,
            unit: String::from(""),
            parser_type: DateParserType::Date,
        };
        assert_eq!(d.determine_rate(), (1, "per second".to_string()))
    }
}
