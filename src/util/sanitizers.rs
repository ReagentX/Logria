pub mod length {
    use regex::bytes::Regex;

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
            let s = self.color_pattern.replace_all(content.as_bytes(), "".as_bytes());
            s.len()
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
}
