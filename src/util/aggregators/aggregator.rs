use crate::util::error::LogriaError;
use serde::{Deserialize, Serialize};

/// Attempts to quicly extrat a float from a string, may have weird effects if numbers
/// are poorly formatted or are immediately next to each other.
pub fn extact_number(message: &str) -> Option<f64> {
    // Result float to parse
    let mut result = String::new();

    // If we have started compiling a float
    let mut in_float = false;

    // For each char, check if it is a sign, digit, or digit separator
    // If it is, flip the float switch, and build the float string
    for (_, char) in message.char_indices() {
        if char.is_digit(10) || char == '.' || char == ',' || char == '-' {
            if !in_float {
                in_float = !in_float;
            }
            if char != ',' {
                result.push(char);
            }
        } else if in_float {
            break;
        }
    }
    result.parse::<f64>().ok()
}

pub trait Aggregator {
    fn update(&mut self, message: &str) -> Result<(), LogriaError>;
    fn messages(&self, n: &usize) -> Vec<String>;
}

#[derive(Eq, PartialEq, Serialize, Deserialize, Debug)]
pub enum AggregationMethod {
    Mean,
    Mode, // Special case of Count, for most_common(1)
    Sum,
    Count,
    Date(String),
    Time(String),
    DateTime(String),
}

#[cfg(test)]
mod tests {
    use super::extact_number;

    #[test]
    fn no_number() {
        let result = extact_number("this is a test");
        assert!(result.is_none());
    }

    #[test]
    fn only_number() {
        let result = extact_number("834234.34");
        assert!(result.unwrap() - 834234.34 == 0.);
    }

    #[test]
    fn only_number_comma() {
        let result = extact_number("834,234.34");
        assert!(result.unwrap() - 834234.34 == 0.);
    }

    #[test]
    fn only_number_multiple_commas() {
        let result = extact_number("834,789,234.34");
        assert!(result.unwrap() - 834789234.34 == 0.);
    }

    #[test]
    fn negative_number() {
        let result = extact_number("test -83,234.34 this is");
        assert!(result.unwrap() + 83234.34 == 0.);
    }

    #[test]
    fn double_negative_number() {
        let result = extact_number("test --83,234.34 this is");
        assert!(result.is_none());
    }

    #[test]
    fn trailing_negative_number() {
        let result = extact_number("test 83,234.34-- this is");
        assert!(result.is_none());
    }

    #[test]
    fn number_period_extra() {
        let result = extact_number("this is a 123.123.123 test");
        assert!(result.is_none());
    }

    #[test]
    fn number_trailing_comma() {
        let result = extact_number("this is a 123.123,123 test");
        // This is actually a bad edge case
        assert!(result.unwrap() - 123.123123 == 0.);
    }

    #[test]
    fn number_trailing_decimal() {
        let result = extact_number("this is a 123.123. test");
        assert!(result.is_none());
    }

    #[test]
    fn one_number_end() {
        let result = extact_number("this is a test 123.4");
        assert!(result.unwrap() - 123.4 == 0.);
    }

    #[test]
    fn one_number_middle() {
        let result = extact_number("this is 123.46 a test");
        assert!(result.unwrap() - 123.46 == 0.);
    }

    #[test]
    fn one_number_start() {
        let result = extact_number("653.12 this is a test");
        assert!(result.unwrap() - 653.12 == 0.);
    }

    #[test]
    fn no_spaces() {
        let result = extact_number("thisis983.12a test");
        assert!(result.unwrap() - 983.12 == 0.);
    }

    #[test]
    fn two_numbers_start_end() {
        let result = extact_number("4.123 this is a test 123.4");
        assert!(result.unwrap() - 4.123 == 0.);
    }

    #[test]
    fn two_numbers_middle() {
        let result = extact_number("this 1337 is 5543 a test");
        assert!(result.unwrap() - 1337. == 0.);
    }
}
