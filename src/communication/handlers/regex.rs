use regex::bytes::Regex;

use super::handler::HanderMethods;
use crate::communication::handlers::user_input::UserInputHandler;
use crate::communication::reader::main::MainWindow;
use crate::constants::cli::patterns::ANSI_COLOR_PATTERN;
use crate::communication::input::input_type::InputType::Normal;

pub struct RegexHandler {
    color_pattern: Regex,
    current_pattern: Option<Regex>,
    input_hander: UserInputHandler,
}

impl RegexHandler {
    /// Process matches, loading the buffer of indexes to matched messages in the main buffer
    /// TODO: possibly async? possibly loading indicator for large jobs?
    pub fn process_matches(&self, window: &mut MainWindow) {
        match &self.current_pattern {
            Some(pattern) => {
                // Start from where we left off to the most recent message
                let buf_range = (window.config.last_index_regexed, window.messages().len());

                // Iterate "forever", skipping to the start and taking up till end-start
                // TODO: Something to indicate progress
                for index in (0..).skip(buf_range.0).take(buf_range.1 - buf_range.0) {
                    if pattern.is_match(&window.messages()[index].as_bytes()) {
                        window.config.matched_rows.push(index);
                    }

                    // Update the last spot so we know where to start next time
                    window.config.last_index_regexed = index + 1;
                }
            }
            None => {
                panic!("Called process with no regex!");
            }
        }
    }

    fn set_pattern(&mut self, window: &mut MainWindow) {
        let pattern = self.input_hander.gather(window);
        self.current_pattern = match Regex::new(&pattern) {
            Ok(regex) => {
                window.write_to_command_line(&format!("Regex with pattern /{}/", pattern));

                // Update the main window's status
                window.config.regex_pattern = Some(pattern);
                Some(regex)
            }
            Err(e) => {
                // TODO: Alert user of invalid regex somehow?
                window.write_to_command_line(&format!("Invalid regex: /{}/ ({})", pattern, e));
                None
            }
        };
        window.set_cli_cursor(Some(ncurses::ACS_VLINE()));
    }

    fn return_to_normal(&mut self, window: &mut MainWindow) {
        self.clear_matches(window);
        window.input_type = Normal;
        window.set_cli_cursor(None);
    }

    fn clear_matches(&mut self, window: &mut MainWindow) {
        self.current_pattern = None;
        window.config.regex_pattern = None;
        window.config.matched_rows = vec![];
        window.config.last_index_regexed = 0;
        window.reset_command_line();
    }
}

impl HanderMethods for RegexHandler {
    fn new() -> RegexHandler {
        RegexHandler {
            color_pattern: Regex::new(ANSI_COLOR_PATTERN).unwrap(),
            current_pattern: None,
            input_hander: UserInputHandler::new(),
        }
    }

    fn recieve_input(&mut self, window: &mut MainWindow, key: i32) {
        match &self.current_pattern {
            Some(_) => match key {
                47 => {
                    self.clear_matches(window);
                    window.set_cli_cursor(None);
                } // enter/return
                27 => self.return_to_normal(window), // esc
                _ => {}
            },
            None => match key {
                10 => {
                   self.set_pattern(window);
                   if self.current_pattern.is_some() {
                       self.process_matches(window);
                   };
                }, // enter/return
                27 => self.return_to_normal(window), // esc
                key => self.input_hander.recieve_input(window, key),
            },
        }
    }
}
