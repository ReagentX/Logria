use std::{borrow::Cow, cmp::max, str::from_utf8};

use regex::bytes::Regex;

use crate::constants::cli::patterns::ANSI_COLOR_PATTERN;

/// Sanitize a filename by replacing or removing characters that are not allowed in filenames
/// across different operating systems (Windows, macOS, Linux)
/// 
/// Uses `Cow` to avoid unnecessary allocations when the filename is already valid.
pub fn sanitize_filename(filename: &str) -> Cow<str> {
    // Characters that are not allowed in filenames on Windows, macOS, or Linux
    // Windows: < > : " | ? * \ /
    // Also includes control characters (0-31) and DEL (127)
    // Reserved names on Windows: CON, PRN, AUX, NUL, COM1-9, LPT1-9

    // Check if we need to trim leading/trailing whitespace and dots
    let trimmed = filename.trim_matches(|c: char| c.is_whitespace() || c == '.');
    let needs_trimming = trimmed.len() != filename.len();

    // Check if any characters need to be replaced
    let has_invalid_chars = trimmed.chars().any(|c| match c {
        '<' | '>' | ':' | '"' | '|' | '?' | '*' | '\\' | '/' => true,
        c if c.is_control() => true,
        _ => false,
    });

    // Check if it's a Windows reserved name
    let upper_name = trimmed.to_uppercase();
    let reserved_names = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    let is_reserved = reserved_names.contains(&upper_name.as_str());

    // Check if it's empty after trimming
    let is_empty = trimmed.is_empty();

    // Check if it's too long
    let is_too_long = trimmed.len() > 255;

    // If no changes are needed, return the original string (no allocation)
    if !needs_trimming && !has_invalid_chars && !is_reserved && !is_empty && !is_too_long {
        return Cow::Borrowed(filename);
    }

    // Use Cow to minimize allocations during the sanitization process
    let mut result: Cow<str> = if needs_trimming {
        Cow::Owned(trimmed.to_string())
    } else {
        Cow::Borrowed(filename)
    };

    // Replace invalid characters if needed
    if has_invalid_chars {
        result = Cow::Owned(
            result
                .chars()
                .map(|c| match c {
                    // Windows/general forbidden characters
                    '<' | '>' | ':' | '"' | '|' | '?' | '*' | '\\' | '/' => '_',
                    // Control characters and DEL
                    c if c.is_control() => '_',
                    // Keep valid characters
                    c => c,
                })
                .collect::<String>(),
        );
    }

    // Handle Windows reserved names by appending underscore
    if is_reserved {
        result = Cow::Owned(format!("{}_", result));
    }

    // Handle empty filename
    if is_empty {
        result = Cow::Borrowed("unnamed_session");
    }

    // Limit length to 255 characters (common filesystem limit)
    if result.len() > 255 {
        let mut truncated = result.into_owned();
        truncated.truncate(255);
        result = Cow::Owned(truncated);
    }

    result
}

pub struct LengthFinder {
    color_pattern: Regex,
}

impl LengthFinder {
    pub fn new() -> LengthFinder {
        LengthFinder {
            color_pattern: Regex::new(ANSI_COLOR_PATTERN).unwrap(),
        }
    }

    /// Returns the length of the string without ANSI color codes, i.e. the
    /// number of visible characters in a string when rendered in a terminal.
    fn get_real_length(&self, content: &str) -> usize {
        self.color_pattern
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
    use std::borrow::Cow;
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
        assert_eq!(sanitize_filename("file.txt"), "file.txt");
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
        assert_eq!(sanitize_filename("..filename.."), "filename");
        assert_eq!(sanitize_filename("\tfilename\t"), "filename");
    }

    #[test]
    fn test_sanitize_filename_reserved_names() {
        assert_eq!(sanitize_filename("CON"), "CON_");
        assert_eq!(sanitize_filename("con"), "con_");
        assert_eq!(sanitize_filename("PRN"), "PRN_");
        assert_eq!(sanitize_filename("COM1"), "COM1_");
        assert_eq!(sanitize_filename("LPT1"), "LPT1_");
    }

    #[test]
    fn test_sanitize_filename_empty() {
        assert_eq!(sanitize_filename(""), "unnamed_session");
        assert_eq!(sanitize_filename("   "), "unnamed_session");
        assert_eq!(sanitize_filename("..."), "unnamed_session");
    }

    #[test]
    fn test_sanitize_filename_long() {
        let long_name = "a".repeat(300);
        let sanitized = sanitize_filename(&long_name);
        assert_eq!(sanitized.len(), 255);
    }

    #[test]
    fn test_sanitize_filename_no_allocation_needed() {
        // These should not require any modifications, demonstrating Cow optimization
        let clean_names = vec![
            "normal_filename",
            "file.txt", 
            "file_123",
            "valid-name.log",
            "test_file_2024.json"
        ];
        
        for name in clean_names {
            let result = sanitize_filename(name);
            assert_eq!(result, name);
            // The function should return efficiently without unnecessary string operations
        }
    }

    #[test]
    fn test_sanitize_filename_cow_borrowed_optimization() {
        // Test that clean filenames return Cow::Borrowed (no allocation)
        let clean_name = "clean_filename.txt";
        let result = sanitize_filename(clean_name);
        
        // Verify the result is correct
        assert_eq!(result, clean_name);
        
        // Verify it's a borrowed reference (no allocation)
        match result {
            Cow::Borrowed(_) => {}, // This is what we want
            Cow::Owned(_) => panic!("Expected Cow::Borrowed for clean filename, got Cow::Owned"),
        }
    }

    #[test]
    fn test_sanitize_filename_cow_owned_when_modified() {
        // Test that modified filenames return Cow::Owned (allocation only when needed)
        let dirty_name = "dirty<>filename.txt";
        let result = sanitize_filename(dirty_name);
        
        // Verify the result is sanitized
        assert_eq!(result, "dirty__filename.txt");
        
        // Verify it's an owned string (allocation was necessary)
        match result {
            Cow::Owned(_) => {}, // This is what we want when modifications are needed
            Cow::Borrowed(_) => panic!("Expected Cow::Owned for modified filename, got Cow::Borrowed"),
        }
    }
}
