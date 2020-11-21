use regex::Regex;

use crate::communication::reader::main::MainWindow;
use crate::constants::cli::patterns::ANSI_COLOR_PATTERN;
use super::handler::HanderMethods;

pub struct RegexHandler {
    color_pattern: Regex, 
}

impl RegexHandler {

}

impl HanderMethods for RegexHandler {
    fn new() -> RegexHandler {
        RegexHandler {
            color_pattern: Regex::new(&ANSI_COLOR_PATTERN).unwrap(),
        }
    }

    fn recieve_input(&mut self, window: &mut MainWindow, key: i32) {
        window.write_to_command_line(&format!("got data in RegexHandler: {}", key))
    }
}
