use std::{
    collections::HashMap,
    error::Error,
    fs::{create_dir_all, read_dir, read_to_string, write},
    path::Path,
    result::Result,
};

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{constants::directories::patterns, util::error::LogriaError};

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Debug)]
pub enum PatternType {
    Split,
    Regex,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Parser {
    pub pattern: String,
    pub pattern_type: PatternType, // Cannot use `type` for the name as it is reserved
    pub name: String,
    pub example: String,
    pub analytics_methods: HashMap<String, String>,
    #[serde(skip_serializing, skip_deserializing)]
    analytics_map: HashMap<String, String>,
    #[serde(skip_serializing, skip_deserializing)]
    analytics: HashMap<String, String>,
    #[serde(skip_serializing, skip_deserializing)]
    num_to_print: i32,
}

impl Parser {
    /// Ensure the proper paths exist
    fn verify_path() {
        let tape_path = patterns();
        if !Path::new(&tape_path).exists() {
            create_dir_all(tape_path).unwrap();
        }
    }

    /// Create an instance of a parser
    pub fn new(
        pattern: String,
        pattern_type: PatternType,
        name: String,
        example: String,
        analytics_methods: HashMap<String, String>,
        num_to_print: Option<i32>,
    ) -> Parser {
        Parser::verify_path();
        Parser {
            pattern,
            pattern_type,
            name,
            example,
            analytics_methods,
            analytics_map: HashMap::new(),
            analytics: HashMap::new(),
            num_to_print: num_to_print.unwrap_or(5),
        }
    }

    /// Create parser file from a Parser struct
    pub fn save(self) -> Result<(), LogriaError> {
        let parser_json = serde_json::to_string_pretty(&self).unwrap();
        let path = format!("{}/{}", patterns(), self.name);

        match write(format!("{}/{}", patterns(), self.name), parser_json) {
            Ok(_) => Ok(()),
            Err(why) => Err(LogriaError::CannotWrite(path, <dyn Error>::to_string(&why))),
        }
    }

    /// Create Parser struct from a parser file
    pub fn load(file_name: &str) -> Result<Parser, LogriaError> {
        let parser_json = match read_to_string(file_name) {
            Ok(json) => json,
            Err(why) => {
                return Err(LogriaError::CannotRead(
                    file_name.to_owned(),
                    <dyn Error>::to_string(&why),
                ))
            }
        };
        let session: Parser = serde_json::from_str(&parser_json).unwrap();
        Ok(session)
    }

    /// Get a list of all available parser configurations
    pub fn list() -> Vec<String> {
        let mut parsers: Vec<String> = read_dir(patterns())
            .unwrap()
            .map(|parser| String::from(parser.unwrap().path().to_str().unwrap()))
            .collect();
        parsers.sort();
        parsers
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
                            .for_each(|value| example.push(format!("{}", value.unwrap().as_str())));
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
                    .for_each(|value| example.push(format!("{}", value)));
            }
        };

        // Validate the size of the generated text
        if example.len() != self.analytics_methods.len() {
            return Err(LogriaError::InvalidExampleSplit(
                example.len(),
                self.analytics_methods.len(),
            ));
        }
        Ok(example)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{Parser, PatternType};
    use crate::constants::directories::patterns;

    #[test]
    fn test_list() {
        let list = Parser::list();
        assert!(list
            .iter()
            .any(|i| i == &format!("{}/{}", patterns(), "Hyphen Separated")))
    }

    #[test]
    fn serialize_deserialize_session() {
        let mut map = HashMap::new();
        map.insert(String::from("Date"), String::from("date"));
        map.insert(String::from("Caller"), String::from("count"));
        map.insert(String::from("Level"), String::from("count"));
        map.insert(String::from("Message"), String::from("sum"));
        let parser = Parser::new(
            String::from(" - "),
            PatternType::Split,
            String::from("Hyphen Separated Copy"),
            String::from("2005-03-19 15:10:26,773 - simple_example - CRITICAL - critical message"),
            map.to_owned(),
            None,
        );
        parser.save().unwrap();

        let file_name = format!("{}/{}", patterns(), "Hyphen Separated Copy");
        let read_parser = Parser::load(&file_name).unwrap();
        let expected_parser = Parser::new(
            String::from(" - "),
            PatternType::Split,
            String::from("Hyphen Separated Copy"),
            String::from("2005-03-19 15:10:26,773 - simple_example - CRITICAL - critical message"),
            map,
            None,
        );
        assert_eq!(read_parser.pattern, expected_parser.pattern);
        assert_eq!(read_parser.pattern_type, expected_parser.pattern_type);
        assert_eq!(read_parser.name, expected_parser.name);
        assert_eq!(
            read_parser.analytics_methods,
            expected_parser.analytics_methods
        );
    }

    #[test]
    fn can_get_regex() {
        let mut map = HashMap::new();
        map.insert(String::from("Remote Host"), String::from("count"));
        map.insert(String::from("User ID"), String::from("count"));
        map.insert(String::from("Username"), String::from("count"));
        map.insert(String::from("Date"), String::from("count"));
        map.insert(String::from("Request"), String::from("count"));
        map.insert(String::from("Status"), String::from("count"));
        map.insert(String::from("Size"), String::from("count"));
        let parser = Parser::new(
            String::from("([^ ]*) ([^ ]*) ([^ ]*) \\[([^]]*)\\] \"([^\"]*)\" ([^ ]*) ([^ ]*)"),
            PatternType::Regex,
            String::from("Common Log Format Test 2"),
            String::from("127.0.0.1 user-identifier frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326"),
            map,
            None
        );
        parser.save().unwrap();

        let file_name = format!("{}/{}", patterns(), "Common Log Format Test 2");
        let read_parser = Parser::load(&file_name);
        let regex = read_parser.unwrap().get_regex();
        assert!(regex.is_ok());
    }

    #[test]
    fn cannot_get_regex() {
        let mut map = HashMap::new();
        map.insert(String::from("Date"), String::from("date"));
        map.insert(String::from("Caller"), String::from("count"));
        map.insert(String::from("Level"), String::from("count"));
        map.insert(String::from("Message"), String::from("sum"));
        let parser = Parser::new(
            String::from(" - "),
            PatternType::Split,
            String::from("Hyphen Separated Test 1"),
            String::from("2005-03-19 15:10:26,773 - simple_example - CRITICAL - critical message"),
            map,
            None,
        );
        parser.save().unwrap();

        let file_name = format!("{}/{}", patterns(), "Hyphen Separated Test 1");
        let parser = Parser::load(&file_name);
        let regex = parser.unwrap().get_regex();
        assert!(regex.is_err());
    }

    #[test]
    fn can_get_example_regex() {
        let mut map = HashMap::new();
        map.insert(String::from("Remote Host"), String::from("count"));
        map.insert(String::from("User ID"), String::from("count"));
        map.insert(String::from("Username"), String::from("count"));
        map.insert(String::from("Date"), String::from("count"));
        map.insert(String::from("Request"), String::from("count"));
        map.insert(String::from("Status"), String::from("count"));
        map.insert(String::from("Size"), String::from("count"));
        let parser = Parser::new(
            String::from("([^ ]*) ([^ ]*) ([^ ]*) \\[([^]]*)\\] \"([^\"]*)\" ([^ ]*) ([^ ]*)"),
            PatternType::Regex,
            String::from("Common Log Format Test 1"),
            String::from("127.0.0.1 user-identifier frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326"),
            map,
            None
        );
        parser.save().unwrap();

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
        map.insert(String::from("Date"), String::from("date"));
        map.insert(String::from("Caller"), String::from("count"));
        map.insert(String::from("Level"), String::from("count"));
        map.insert(String::from("Message"), String::from("sum"));
        let parser = Parser::new(
            String::from(" - "),
            PatternType::Split,
            String::from("Hyphen Separated Test 2"),
            String::from("2005-03-19 15:10:26,773 - simple_example - CRITICAL - critical message"),
            map,
            None,
        );
        parser.save().unwrap();

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
