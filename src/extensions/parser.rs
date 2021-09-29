use std::{
    collections::HashMap,
    error::Error,
    fs::{create_dir_all, read_dir, read_to_string, remove_file, write},
    path::Path,
    result::Result,
};

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{
    constants::directories::patterns,
    extensions::extension::ExtensionMethods,
    util::{
        aggregators::aggregator::{AggregationMethod, Aggregator},
        error::LogriaError,
    },
};

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Debug)]
pub enum PatternType {
    Split,
    Regex,
}

#[derive(Serialize, Deserialize)]
pub struct Parser {
    pub pattern: String,
    pub pattern_type: PatternType, // Cannot use `type` for the name as it is reserved
    pub example: String,
    pub order: Vec<String>,
    pub aggregation_methods: HashMap<String, AggregationMethod>,
    #[serde(skip_serializing, skip_deserializing)]
    pub aggregator_map: HashMap<String, Box<dyn Aggregator>>,
}

impl ExtensionMethods for Parser {
    /// Ensure the proper paths exist
    fn verify_path() {
        let tape_path = patterns();
        if !Path::new(&tape_path).exists() {
            create_dir_all(tape_path).unwrap();
        }
    }

    /// Create parser file from a Parser struct
    fn save(self, file_name: &str) -> Result<(), LogriaError> {
        let parser_json = serde_json::to_string_pretty(&self).unwrap();
        let path = format!("{}/{}", patterns(), file_name);

        match write(format!("{}/{}", patterns(), file_name), parser_json) {
            Ok(_) => Ok(()),
            Err(why) => Err(LogriaError::CannotWrite(path, <dyn Error>::to_string(&why))),
        }
    }

    /// Delete the path for a fully qualified session filename
    fn del(items: &[usize]) -> Result<(), LogriaError> {
        // Iterate through each `i` in `items` and remove the item at list index `i`
        let files = Parser::list_full();
        for i in items {
            if i >= &files.len() {
                break;
            }
            let file_name = &files[*i];
            match remove_file(file_name) {
                Ok(_) => {}
                Err(why) => {
                    return Err(LogriaError::CannotRemove(
                        file_name.to_owned(),
                        <dyn Error>::to_string(&why),
                    ))
                }
            }
        }
        Ok(())
    }

    /// Get a list of all available parser configurations with fully qualified paths
    fn list_full() -> Vec<String> {
        Parser::verify_path();
        let mut parsers: Vec<String> = read_dir(patterns())
            .unwrap()
            .map(|parser| String::from(parser.unwrap().path().to_str().unwrap()))
            .collect();
        parsers.sort();
        parsers
    }

    /// Get a list of all available parser configurations for display purposes
    fn list_clean() -> Vec<String> {
        Parser::verify_path();
        let mut parsers: Vec<String> = read_dir(patterns())
            .unwrap()
            .map(|parser| {
                String::from(
                    parser
                        .unwrap()
                        .path()
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap(),
                )
            })
            .collect();
        parsers.sort();
        parsers
    }
}

impl Parser {
    /// Create an instance of a parser
    pub fn new(
        pattern: String,
        pattern_type: PatternType,
        example: String,
        order: Vec<String>,
        aggregation_methods: HashMap<String, AggregationMethod>,
    ) -> Parser {
        Parser::verify_path();
        Parser {
            pattern,
            pattern_type,
            example,
            order,
            aggregation_methods,
            aggregator_map: HashMap::new(),
        }
    }

    /// Create Parser struct from a parser file
    pub fn load(file_name: &str) -> Result<Parser, LogriaError> {
        match read_to_string(file_name) {
            Ok(json) => match serde_json::from_str(&json) {
                Ok(parser) => Ok(parser),
                Err(why) => Err(LogriaError::InvalidParserState(why.to_string())),
            },
            Err(why) => Err(LogriaError::CannotRead(
                file_name.to_owned(),
                why.to_string(),
            )),
        }
    }

    pub fn get_regex(&self) -> Result<Regex, LogriaError> {
        if self.pattern_type == PatternType::Regex {
            match Regex::new(&self.pattern) {
                Ok(pattern) => Ok(pattern),
                Err(why) => Err(LogriaError::InvalidRegex(why, self.pattern.to_owned())),
            }
        } else {
            Err(LogriaError::WrongParserType)
        }
    }

    pub fn get_example(&self) -> std::result::Result<Vec<String>, LogriaError> {
        let mut example: Vec<String> = vec![];
        match self.pattern_type {
            PatternType::Regex => match self.get_regex() {
                Ok(regex) => {
                    if let Some(captures) = regex.captures(&self.example) {
                        captures
                            .iter()
                            .skip(1)
                            .for_each(|value| example.push(value.unwrap().as_str().to_string()));
                    } else {
                        {
                            return Err(LogriaError::InvalidExampleRegex(self.pattern.to_owned()));
                        }
                    }
                }
                Err(why) => {
                    return Err(why);
                }
            },
            PatternType::Split => {
                self.example
                    .split(&self.pattern)
                    .collect::<Vec<&str>>()
                    .iter()
                    .for_each(|value| example.push(value.to_string()));
            }
        };

        // Validate the size of the generated text
        if example.len() != self.aggregation_methods.len() {
            return Err(LogriaError::InvalidExampleSplit(
                example.len(),
                self.aggregation_methods.len(),
            ));
        }
        Ok(example)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        constants::directories::patterns,
        extensions::{
            extension::ExtensionMethods,
            parser::{AggregationMethod, Parser, PatternType},
        },
    };

    #[test]
    fn test_list_full() {
        // Create a parser for use by this test
        let mut map = HashMap::new();
        map.insert(
            String::from("Date"),
            AggregationMethod::Date(String::from("[year]-[month]-[day]")),
        );
        map.insert(String::from("Method"), AggregationMethod::Count);
        map.insert(String::from("Level"), AggregationMethod::Count);
        map.insert(String::from("Message"), AggregationMethod::Sum);
        let parser = Parser::new(
            String::from(" - "),
            PatternType::Split,
            String::from("2005-03-19 15:10:26,773 - simple_example - CRITICAL - critical message"),
            vec![
                "Date".to_string(),
                "Message".to_string(),
                "Level".to_string(),
                "Message".to_string(),
            ],
            map,
        );
        parser.save("Hyphen Separated Test 3").unwrap();

        let list = Parser::list_full();
        assert!(list
            .iter()
            .any(|i| i == &format!("{}/{}", patterns(), "Hyphen Separated Test 3")))
    }

    #[test]
    fn test_list_clean() {
        // Create a parser for use by this test
        let mut map = HashMap::new();
        map.insert(
            String::from("Date"),
            AggregationMethod::Date(String::from("[year]-[month]-[day]")),
        );
        map.insert(String::from("Method"), AggregationMethod::Count);
        map.insert(String::from("Level"), AggregationMethod::Count);
        map.insert(String::from("Message"), AggregationMethod::Sum);
        let parser = Parser::new(
            String::from(" - "),
            PatternType::Split,
            String::from("2005-03-19 15:10:26,773 - simple_example - CRITICAL - critical message"),
            vec![
                "Date".to_string(),
                "Message".to_string(),
                "Level".to_string(),
                "Message".to_string(),
            ],
            map,
        );
        parser.save("Hyphen Separated Test 3").unwrap();

        let list = Parser::list_clean();
        assert!(list.iter().any(|i| i == "Hyphen Separated Test 3"))
    }

    #[test]
    fn serialize_deserialize_session() {
        let mut map = HashMap::new();
        map.insert(
            String::from("Date"),
            AggregationMethod::Date(String::from("[year]-[month]-[day]")),
        );
        map.insert(String::from("Method"), AggregationMethod::Count);
        map.insert(String::from("Level"), AggregationMethod::Count);
        map.insert(String::from("Message"), AggregationMethod::Sum);
        let mut map2 = HashMap::new();
        map2.insert(
            String::from("Date"),
            AggregationMethod::Date(String::from("[year]-[month]-[day]")),
        );
        map2.insert(String::from("Method"), AggregationMethod::Count);
        map2.insert(String::from("Level"), AggregationMethod::Count);
        map2.insert(String::from("Message"), AggregationMethod::Sum);
        let parser = Parser::new(
            String::from(" - "),
            PatternType::Split,
            String::from("2005-03-19 15:10:26,773 - simple_example - CRITICAL - critical message"),
            vec![
                "Date".to_string(),
                "Message".to_string(),
                "Level".to_string(),
                "Message".to_string(),
            ],
            map2,
        );
        parser.save("Hyphen Separated Test 2").unwrap();

        let file_name = format!("{}/{}", patterns(), "Hyphen Separated Test 2");
        let read_parser = Parser::load(&file_name).unwrap();
        let expected_parser = Parser::new(
            String::from(" - "),
            PatternType::Split,
            String::from("2005-03-19 15:10:26,773 - simple_example - CRITICAL - critical message"),
            vec![
                "Date".to_string(),
                "Message".to_string(),
                "Level".to_string(),
                "Message".to_string(),
            ],
            map,
        );
        assert_eq!(read_parser.pattern, expected_parser.pattern);
        assert_eq!(read_parser.pattern_type, expected_parser.pattern_type);
        assert_eq!(
            read_parser.aggregation_methods,
            expected_parser.aggregation_methods
        );
    }

    #[test]
    fn serialize_deserialize_session_datetime() {
        let mut map = HashMap::new();
        map.insert(
            String::from("DateTime"),
            AggregationMethod::DateTime(String::from("[year]-[month]-[day] [hour]:[month]:[second]")),
        );
        map.insert(String::from("Method"), AggregationMethod::Count);
        map.insert(String::from("Level"), AggregationMethod::Count);
        map.insert(String::from("Message"), AggregationMethod::Sum);
        let mut map2 = HashMap::new();
        map2.insert(
            String::from("DateTime"),
            AggregationMethod::DateTime(String::from("[year]-[month]-[day] [hour]:[month]:[second]")),
        );
        map2.insert(String::from("Method"), AggregationMethod::Count);
        map2.insert(String::from("Level"), AggregationMethod::Count);
        map2.insert(String::from("Message"), AggregationMethod::Sum);
        let parser = Parser::new(
            String::from(" - "),
            PatternType::Split,
            String::from("2005-03-19 15:10:26,773 - simple_example - CRITICAL - critical message"),
            vec![
                "Date".to_string(),
                "Message".to_string(),
                "Level".to_string(),
                "Message".to_string(),
            ],
            map2,
        );
        parser.save("Hyphen Separated Test 2").unwrap();

        let file_name = format!("{}/{}", patterns(), "Hyphen Separated Test 2");
        let read_parser = Parser::load(&file_name).unwrap();
        let expected_parser = Parser::new(
            String::from(" - "),
            PatternType::Split,
            String::from("2005-03-19 15:10:26,773 - simple_example - CRITICAL - critical message"),
            vec![
                "Date".to_string(),
                "Message".to_string(),
                "Level".to_string(),
                "Message".to_string(),
            ],
            map,
        );
        assert_eq!(read_parser.pattern, expected_parser.pattern);
        assert_eq!(read_parser.pattern_type, expected_parser.pattern_type);
        assert_eq!(
            read_parser.aggregation_methods,
            expected_parser.aggregation_methods
        );
    }

    #[test]
    fn can_get_regex() {
        let mut map = HashMap::new();
        map.insert(String::from("Remote Host"), AggregationMethod::Count);
        map.insert(String::from("User ID"), AggregationMethod::Count);
        map.insert(String::from("Username"), AggregationMethod::Count);
        map.insert(String::from("Date"), AggregationMethod::Count);
        map.insert(String::from("Request"), AggregationMethod::Count);
        map.insert(String::from("Status"), AggregationMethod::Count);
        map.insert(String::from("Size"), AggregationMethod::Count);
        let parser = Parser::new(
            String::from("([^ ]*) ([^ ]*) ([^ ]*) \\[([^]]*)\\] \"([^\"]*)\" ([^ ]*) ([^ ]*)"),
            PatternType::Regex,
            String::from("127.0.0.1 user-identifier frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326"),
            vec!["Remote Host".to_string(), "User ID".to_string(), "Username".to_string(), "Date".to_string(), "Request".to_string(), "Status".to_string(), "Size".to_string()],
            map,
        );
        parser.save("Common Log Format Test 2").unwrap();

        let file_name = format!("{}/{}", patterns(), "Common Log Format Test 2");
        let read_parser = Parser::load(&file_name);
        let regex = read_parser.unwrap().get_regex();
        assert!(regex.is_ok());
    }

    #[test]
    fn cannot_get_regex() {
        let mut map = HashMap::new();
        map.insert(
            String::from("Date"),
            AggregationMethod::Date(String::from("[year]-[month]-[day]")),
        );
        map.insert(String::from("Method"), AggregationMethod::Count);
        map.insert(String::from("Level"), AggregationMethod::Count);
        map.insert(String::from("Message"), AggregationMethod::Sum);
        let parser = Parser::new(
            String::from(" - "),
            PatternType::Split,
            String::from("2005-03-19 15:10:26,773 - simple_example - CRITICAL - critical message"),
            vec![
                "Date".to_string(),
                "Message".to_string(),
                "Level".to_string(),
                "Message".to_string(),
            ],
            map,
        );
        parser.save("Hyphen Separated Test 1").unwrap();

        let file_name = format!("{}/{}", patterns(), "Hyphen Separated Test 1");
        let parser = Parser::load(&file_name);
        let regex = parser.unwrap().get_regex();
        assert!(regex.is_err());
    }

    #[test]
    fn can_get_example_regex() {
        let mut map = HashMap::new();
        map.insert(String::from("Remote Host"), AggregationMethod::Count);
        map.insert(String::from("User ID"), AggregationMethod::Count);
        map.insert(String::from("Username"), AggregationMethod::Count);
        map.insert(String::from("Date"), AggregationMethod::Count);
        map.insert(String::from("Request"), AggregationMethod::Count);
        map.insert(String::from("Status"), AggregationMethod::Count);
        map.insert(String::from("Size"), AggregationMethod::Count);
        let parser = Parser::new(
            String::from("([^ ]*) ([^ ]*) ([^ ]*) \\[([^]]*)\\] \"([^\"]*)\" ([^ ]*) ([^ ]*)"),
            PatternType::Regex,
            String::from("127.0.0.1 user-identifier frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326"),
            vec!["Remote Host".to_string(), "User ID".to_string(), "Username".to_string(), "Date".to_string(), "Request".to_string(), "Status".to_string(), "Size".to_string()],
            map,
        );
        parser.save("Common Log Format Test 1").unwrap();

        let file_name = format!("{}/{}", patterns(), "Common Log Format Test 1");
        let parser = Parser::load(&file_name);
        assert_eq!(
            parser.unwrap().get_example().unwrap(),
            vec![
                String::from("127.0.0.1"),
                String::from("user-identifier"),
                String::from("frank"),
                String::from("10/Oct/2000:13:55:36 -0700"),
                String::from("GET /apache_pb.gif HTTP/1.0"),
                String::from("200"),
                String::from("2326")
            ]
        );
    }

    #[test]
    fn can_get_example_split() {
        let mut map = HashMap::new();
        map.insert(
            String::from("Date"),
            AggregationMethod::Date(String::from("[year]-[month]-[day]")),
        );
        map.insert(String::from("Method"), AggregationMethod::Count);
        map.insert(String::from("Level"), AggregationMethod::Count);
        map.insert(String::from("Message"), AggregationMethod::Sum);
        let parser = Parser::new(
            String::from(" - "),
            PatternType::Split,
            String::from("2005-03-19 15:10:26,773 - simple_example - CRITICAL - critical message"),
            vec![
                "Date".to_string(),
                "Message".to_string(),
                "Level".to_string(),
                "Message".to_string(),
            ],
            map,
        );
        parser.save("Hyphen Separated Test 2").unwrap();

        let file_name = format!("{}/{}", patterns(), "Hyphen Separated Test 2");
        let parser = Parser::load(&file_name);
        assert_eq!(
            parser.unwrap().get_example().unwrap(),
            vec![
                String::from("2005-03-19 15:10:26,773"),
                String::from("simple_example"),
                String::from("CRITICAL"),
                String::from("critical message")
            ]
        );
    }
}
