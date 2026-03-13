use std::io::Result;

use regex::bytes::Regex;

use super::{handler::Handler, processor::update_progress, user_input::UserInputHandler};
use crate::{
    communication::{input::InputType::Normal, reader::MainWindow},
    constants::cli::{cli_chars::NORMAL_STR, patterns::ANSI_COLOR_REGEX},
};

/// Shared state and logic for regex filtering and highlight searching.
///
/// Both `RegexHandler` and `HighlightHandler` compose this struct
/// to avoid duplicating pattern matching, match processing, and
/// cleanup logic.
pub struct SearchState {
    pub current_pattern: Option<Regex>,
    pub input_handler: UserInputHandler,
}

impl SearchState {
    pub fn new() -> Self {
        SearchState {
            current_pattern: None,
            input_handler: UserInputHandler::new(),
        }
    }

    /// Test a message to see if it matches the pattern while also escaping the color code
    pub fn test(&self, message: &str) -> bool {
        let clean_message = ANSI_COLOR_REGEX.replace_all(message.as_bytes(), "".as_bytes());
        match &self.current_pattern {
            Some(pattern) => pattern.is_match(&clean_message),
            None => panic!("Match called with no pattern!"),
        }
    }

    /// Save the user input pattern to the main window config
    pub fn set_pattern(&mut self, window: &mut MainWindow, mode_name: &str) -> Result<()> {
        let pattern = match self.input_handler.gather(window) {
            Ok(pattern) => pattern,
            Err(why) => panic!("Unable to gather text: {why:?}"),
        };

        self.current_pattern = match Regex::new(&pattern) {
            Ok(regex) => {
                window.config.current_status =
                    Some(format!("{mode_name} with pattern /{pattern}/"));
                window.write_status()?;

                // Update the main window's regex
                window.config.regex_pattern = Some(regex.clone());
                Some(regex)
            }
            Err(e) => {
                window.write_to_command_line(&format!("Invalid regex: /{pattern}/ ({e})"))?;
                None
            }
        };
        window.set_cli_cursor(Some(NORMAL_STR))?;
        window.config.highlight_match = true;
        Ok(())
    }

    /// Process matches, loading the buffer of indexes to matched messages in the main buffer
    pub fn process_matches(&self, window: &mut MainWindow) -> Result<()> {
        let mut wrote_progress = false;
        if self.current_pattern.is_some() {
            // Start from where we left off to the most recent message
            let start = window.config.last_index_regexed;
            let end = window.messages().len();

            for index in start..end {
                if self.test(&window.messages()[index]) {
                    window.config.matched_rows.push(index);
                }

                // Update the user interface with the current state
                wrote_progress = update_progress(window, start, end, index)?;

                // Update the last spot so we know where to start next time
                window.config.last_index_regexed = index + 1;
            }
            if wrote_progress {
                window.write_status()?;
            }
        }
        Ok(())
    }

    /// Clear the matched messages from the message buffer
    pub fn clear_matches(&mut self, window: &mut MainWindow) -> Result<()> {
        self.current_pattern = None;
        window.config.regex_pattern = None;
        window.config.matched_rows.clear();
        window.config.last_index_regexed = 0;
        window.config.highlight_match = false;
        window.reset_command_line()?;
        Ok(())
    }

    /// Finish returning to normal mode after clear_matches has been called
    pub fn finish_return_to_normal(&mut self, window: &mut MainWindow) -> Result<()> {
        window.config.current_status = None;
        window.update_input_type(Normal)?;
        window.set_cli_cursor(None)?;
        self.input_handler.gather(window)?;
        window.redraw()?;
        Ok(())
    }

    /// Return the app to a normal input state
    pub fn return_to_normal(&mut self, window: &mut MainWindow) -> Result<()> {
        self.clear_matches(window)?;
        self.finish_return_to_normal(window)
    }
}
