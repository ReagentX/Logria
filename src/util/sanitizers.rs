pub mod length {
    use regex::bytes::Regex;
    use std::str::from_utf8;

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
        pub fn get_real_length(&self, content: &str) -> usize {
            self.color_pattern
                .split(content.as_bytes())
                .filter_map(|s| from_utf8(s).ok())
                .map(|s| s.chars().count())
                .sum()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::length::LengthFinder;

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
        assert_eq!("\x1b[0m word \x1b[32m", content);
    }

    #[test]
    fn test_length_wide_chars() {
        let l = LengthFinder::new();
        let content = "\x1b[0m█四░\x1b[32m█四░";
        assert_eq!(l.get_real_length(content), 6);
        assert_eq!("\x1b[0m█四░\x1b[32m█四░", content);
    }
}
