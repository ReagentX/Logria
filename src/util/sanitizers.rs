use std::{cmp::max, collections::HashSet, str::from_utf8, sync::LazyLock};

use crate::constants::cli::patterns::ANSI_COLOR_REGEX;

/// Characters disallowed in a filename
static FILENAME_DISALLOWED_CHARS: LazyLock<HashSet<char>> =
    LazyLock::new(|| HashSet::from(['*', '"', '/', '\\', '<', '>', ':', '|', '?', '.']));
/// The character to replace disallowed chars with
const FILENAME_REPLACEMENT_CHAR: char = '_';

/// Remove unsafe chars in [this list](FILENAME_DISALLOWED_CHARS).
///
/// Does not need to use a `Cow` for optimization because the source is always generated based on chat data
/// so there is no opportunity for the original input to be passed in from another borrow.
pub fn sanitize_filename(filename: &str) -> String {
    filename
        .trim()
        .chars()
        .map(|letter| {
            if letter.is_control() || FILENAME_DISALLOWED_CHARS.contains(&letter) {
                FILENAME_REPLACEMENT_CHAR
            } else {
                letter
            }
        })
        .take(255)
        .collect()
}

pub struct LengthFinder {}

impl LengthFinder {
    pub fn new() -> LengthFinder {
        LengthFinder {}
    }

    /// Returns the length of the string without ANSI color codes, i.e. the
    /// number of visible characters in a string when rendered in a terminal.
    fn get_real_length(&self, content: &str) -> usize {
        ANSI_COLOR_REGEX
            .split(content.as_bytes())
            .filter_map(|s| from_utf8(s).ok())
            .map(|s| s.chars().count())
            .sum()
    }

    /// Given a string to render and the terminal width, return a tuple of the number of rows it
    /// would take to display the string and the real length of the string.
    pub fn get_rows_and_length(&self, content: &str, terminal_width: usize) -> (usize, usize) {
        let length = self.get_real_length(content);
        (max(1, (length).div_ceil(terminal_width)), length)
    }
}

#[cfg(test)]
mod tests {
    use crate::util::sanitizers::{LengthFinder, sanitize_filename};

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

    #[test]
    fn test_sanitize_filename_clean() {
        assert_eq!(sanitize_filename("normal_filename"), "normal_filename");
        assert_eq!(sanitize_filename("file.txt"), "file_txt");
        assert_eq!(sanitize_filename("file_123"), "file_123");
    }

    #[test]
    fn test_sanitize_filename_invalid_chars() {
        assert_eq!(sanitize_filename("file<>name"), "file__name");
        assert_eq!(sanitize_filename("file:name"), "file_name");
        assert_eq!(sanitize_filename("file\"name"), "file_name");
        assert_eq!(sanitize_filename("file|name"), "file_name");
        assert_eq!(sanitize_filename("file?name"), "file_name");
        assert_eq!(sanitize_filename("file*name"), "file_name");
        assert_eq!(sanitize_filename("file\\name"), "file_name");
        assert_eq!(sanitize_filename("file/name"), "file_name");
    }

    #[test]
    fn test_sanitize_filename_control_chars() {
        assert_eq!(sanitize_filename("file\x00name"), "file_name");
        assert_eq!(sanitize_filename("file\x1fname"), "file_name");
        assert_eq!(sanitize_filename("file\x7fname"), "file_name");
    }

    #[test]
    fn test_sanitize_filename_trim() {
        assert_eq!(sanitize_filename("  filename  "), "filename");
        assert_eq!(sanitize_filename("..filename.."), "__filename__");
        assert_eq!(sanitize_filename("\tfilename\t"), "filename");
    }

    #[test]
    fn test_sanitize_filename_long() {
        let long_name = "a".repeat(300);
        let sanitized = sanitize_filename(&long_name);
        assert_eq!(sanitized.len(), 255);
    }
}
