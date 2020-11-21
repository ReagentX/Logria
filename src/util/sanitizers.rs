pub mod length { 
    use regex::Regex;
    use crate::constants::cli::patterns::ANSI_COLOR_PATTERN;

    pub struct LengthFinder {
        color_pattern: Regex, 
    }
    
    impl LengthFinder {
        pub fn new() -> LengthFinder {
            LengthFinder {
                color_pattern: Regex::new(&ANSI_COLOR_PATTERN).unwrap(),
            }
        }
    
        pub fn get_real_length(&self, content: &str) -> usize {
            let new_str = String::from(content);
            self.color_pattern.replace_all(&new_str, "").len()
        }
    }
    
}
