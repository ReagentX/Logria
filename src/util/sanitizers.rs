use regex::bytes::Regex;
use std::{cmp::max, str::from_utf8};

use crate::constants::cli::patterns::ANSI_COLOR_PATTERN;

pub struct LengthFinder {
    color_pattern: Regex,
}

impl LengthFinder {
    pub fn new() -> LengthFinder {
        LengthFinder {
            color_pattern: Regex::new(ANSI_COLOR_PATTERN).unwrap(),
        }
    }

    /// Returns the length of the string without ANSI color codes, i.e. the real length of a string when rendered in a terminal.
    fn get_real_length(&self, content: &str) -> usize {
        self.color_pattern
            .split(content.as_bytes())
            .filter_map(|s| from_utf8(s).ok())
            .map(|s| s.chars().count())
            .sum()
    }

    /// Given a string's real length and the terminal width, return the number of rows it would take to display the string.
    pub fn get_rows_and_length(&self, content: &str, terminal_width: usize) -> (usize, usize) {
        let length = self.get_real_length(content);
        (max(1, (length).div_ceil(terminal_width)), length)
    }
}

#[cfg(test)]
mod tests {
    use crate::util::sanitizers::LengthFinder;

    #[test]
    fn test_length_clean() {
        let l = LengthFinder::new();
        assert_eq!(l.get_real_length("word"), 4);
    }

    #[test]
    fn test_length_dirty() {
        let l = LengthFinder::new();
        let content = "\x1b[0m word \x1b[32m";
        assert_eq!(l.get_real_length(content), 6);
    }

    #[test]
    fn test_length_wide_chars() {
        let l = LengthFinder::new();
        let content = "\x1b[0m█四░\x1b[32m█四░";
        assert_eq!(l.get_real_length(content), 6);
    }

    #[test]
    fn test_row_length_clean() {
        let l = LengthFinder::new();
        let (rows, length) = l.get_rows_and_length("word", 10);
        assert_eq!(rows, 1);
        assert_eq!(length, 4);
    }

    #[test]
    fn test_row_length_dirty() {
        let l = LengthFinder::new();
        let content = "\x1b[0m word \x1b[32m";
        let (rows, length) = l.get_rows_and_length(content, 4);
        assert_eq!(rows, 2);
        assert_eq!(length, 6);
    }

    #[test]
    fn test_row_length_wide_chars() {
        let l = LengthFinder::new();
        let content = "\x1b[0m█四░\x1b[32m█四░";
        let (rows, length) = l.get_rows_and_length(content, 10);
        assert_eq!(rows, 1);
        assert_eq!(length, 6);
    }
}
