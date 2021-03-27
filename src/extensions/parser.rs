use std::{
    collections::HashMap,
    error::Error,
    fs::{read_dir, read_to_string, write, create_dir_all},
    path::Path,
};

use serde::{Deserialize, Serialize};

use crate::constants::{directories::patterns};

#[derive(Serialize, Deserialize, Debug)]
pub struct Parser {
    pattern: String,
    pattern_type: String, // Cannot use `type` for the name as it is reserved
    name: String,
    example: String,
    analytics_methods: HashMap<String, String>,
    #[serde(skip_serializing, skip_deserializing)]
    analytics_map: HashMap<String, String>,
    #[serde(skip_serializing, skip_deserializing)]
    analytics: HashMap<String, String>,
    #[serde(skip_serializing, skip_deserializing)]
    num_to_print: i32,
}

impl Parser {
    // Ensure the proper paths exist
    pub fn verify_path() {
        let tape_path = patterns();
        if !Path::new(&tape_path).exists() {
            create_dir_all(tape_path).unwrap();
        } 
    }

    /// Create an instance of a parser
    fn new(
        pattern: String,
        pattern_type: String,
        name: String,
        example: String,
        analytics_methods: HashMap<String, String>,
        num_to_print: Option<i32>,
    ) -> Parser {
        Parser::verify_path();
        Parser {
            pattern: pattern,
            pattern_type: pattern_type,
            name: name,
            example: example,
            analytics_methods: analytics_methods,
            analytics_map: HashMap::new(),
            analytics: HashMap::new(),
            num_to_print: match num_to_print {
                Some(num) => num,
                None => 5,
            },
        }
    }

    /// Create parser file from a Parser struct
    fn save(self) {
        let parser_json = serde_json::to_string_pretty(&self).unwrap();
        let path = format!("{}/{}", patterns(), self.name);

        match write(format!("{}/{}", patterns(), self.name), parser_json) {
            Ok(_) => {}
            Err(why) => panic!("Couldn't write {:?}: {}", path, Error::to_string(&why)),
        }
    }

    /// Create Parser struct from a parser file
    fn load(file_name: &str) -> Parser {
        let path = format!("{}/{}", patterns(), file_name);
        let parser_json = match read_to_string(path) {
            Ok(json) => json,
            Err(why) => panic!("Couldn't open {:?}: {}", patterns(), Error::to_string(&why)),
        };
        let session: Parser = serde_json::from_str(&parser_json).unwrap();
        session
    }

    /// Get a list of all available parser configurations
    fn list() -> Vec<String> {
        let mut parsers: Vec<String> = read_dir(patterns())
            .unwrap()
            .map(|parser| String::from(parser.unwrap().path().to_str().unwrap()))
            .collect();
        parsers.sort();
        parsers
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::Parser;
    use crate::constants::directories::patterns;

    #[test]
    fn test_list() {
        let list = Parser::list();
        assert!(list
            .iter()
            .any(|i| i == &format!("{}/{}", patterns(), "Hyphen Separated")))
    }

    #[test]
    fn serialize_session() {
        let mut map = HashMap::new();
        map.insert(String::from("Date"), String::from("date"));
        map.insert(String::from("Caller"), String::from("count"));
        map.insert(String::from("Level"), String::from("count"));
        map.insert(String::from("Message"), String::from("sum"));
        let parser = Parser::new(
            String::from(" - "),
            String::from("split"),
            String::from("Hyphen Separated"),
            String::from("2005-03-19 15:10:26,773 - simple_example - CRITICAL - critical message"),
            map,
            None,
        );
        parser.save()
    }

    #[test]
    fn deserialize_session() {
        let read_parser = Parser::load("Hyphen Separated Copy");
        let mut expected_map = HashMap::new();
        expected_map.insert(String::from("Date"), String::from("date"));
        expected_map.insert(String::from("Caller"), String::from("count"));
        expected_map.insert(String::from("Level"), String::from("count"));
        expected_map.insert(String::from("Message"), String::from("sum"));
        let expected_parser = Parser::new(
            String::from(" - "),
            String::from("split"),
            String::from("Hyphen Separated Copy"),
            String::from("2005-03-19 15:10:26,773 - simple_example - CRITICAL - critical message"),
            expected_map,
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
}
