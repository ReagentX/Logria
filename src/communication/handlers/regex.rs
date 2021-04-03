use crossterm::{event::KeyCode, Result};
use regex::bytes::Regex;

use super::{handler::HanderMethods, processor::ProcessorMethods};
use crate::{
    communication::{
        handlers::user_input::UserInputHandler, input::input_type::InputType::Normal,
        reader::main::MainWindow,
    },
    constants::cli::{cli_chars::NORMAL_CHAR, patterns::ANSI_COLOR_PATTERN},
    ui::scroll,
};

pub struct RegexHandler {
    color_pattern: Regex,
    current_pattern: Option<Regex>,
    input_handler: UserInputHandler,
}

impl RegexHandler {
    /// Test a message to see if it matches the pattern while also escaping the color code
    fn test(&self, message: &str) -> bool {
        // TODO: Possibly without the extra allocation here?
        let clean_message = self
            .color_pattern
            .replace_all(message.as_bytes(), "".as_bytes());
        match &self.current_pattern {
            Some(pattern) => pattern.is_match(&clean_message),
            None => panic!("Match called with no pattern!"),
        }
    }

    /// Save the user input pattern to the main window conig
    fn set_pattern(&mut self, window: &mut MainWindow) -> Result<()> {
        let pattern = match self.input_handler.gather(window) {
            Ok(pattern) => pattern,
            Err(why) => panic!("Unable to gather text: {:?}", why),
        };

        self.current_pattern = match Regex::new(&pattern) {
            Ok(regex) => {
                window.write_to_command_line(&format!("Regex with pattern /{}/", pattern))?;

                // Update the main window's regex
                window.config.regex_pattern = Some(regex.to_owned());
                Some(regex)
            }
            Err(e) => {
                // TODO: Alert user of invalid regex somehow?
                window.write_to_command_line(&format!("Invalid regex: /{}/ ({})", pattern, e))?;
                None
            }
        };
        window.set_cli_cursor(Some(NORMAL_CHAR))?;
        window.config.highlight_match = true;
        Ok(())
    }
}

impl ProcessorMethods for RegexHandler {
    /// Process matches, loading the buffer of indexes to matched messages in the main buffer
    fn process_matches(&self, window: &mut MainWindow) -> Result<()> {
        // TODO: Possibly async? Possibly loading indicator for large jobs?
        match &self.current_pattern {
            Some(_) => {
                // Start from where we left off to the most recent message
                let buf_range = (window.config.last_index_regexed, window.messages().len());

                // Iterate "forever", skipping to the start and taking up till end-start
                // TODO: Something to indicate progress
                for index in (0..).skip(buf_range.0).take(buf_range.1 - buf_range.0) {
                    if self.test(&window.messages()[index]) {
                        window.config.matched_rows.push(index);
                    }

                    // Update the last spot so we know where to start next time
                    window.config.last_index_regexed = index + 1;
                }
            }
            None => {
                panic!("Called process with no regex!");
            }
        };
        Ok(())
    }

    /// Return the app to a normal input state
    fn return_to_normal(&mut self, window: &mut MainWindow) -> Result<()> {
        self.clear_matches(window)?;
        window.input_type = Normal;
        window.set_cli_cursor(None)?;
        window.redraw()?;
        Ok(())
    }

    /// Clear the matched messages from the message buffer
    fn clear_matches(&mut self, window: &mut MainWindow) -> Result<()> {
        self.current_pattern = None;
        window.config.regex_pattern = None;
        window.config.matched_rows.clear();
        window.config.last_index_regexed = 0;
        window.config.highlight_match = false;
        window.reset_command_line()?;
        Ok(())
    }
}

impl HanderMethods for RegexHandler {
    fn new() -> RegexHandler {
        RegexHandler {
            color_pattern: Regex::new(ANSI_COLOR_PATTERN).unwrap(),
            current_pattern: None,
            input_handler: UserInputHandler::new(),
        }
    }

    fn recieve_input(&mut self, window: &mut MainWindow, key: KeyCode) -> Result<()> {
        match &self.current_pattern {
            Some(_) => match key {
                // Scroll
                KeyCode::Down => scroll::down(window),
                KeyCode::Up => scroll::up(window),
                KeyCode::Left => scroll::top(window),
                KeyCode::Right => scroll::bottom(window),
                KeyCode::Home => scroll::top(window),
                KeyCode::End => scroll::bottom(window),
                KeyCode::PageUp => scroll::pg_down(window),
                KeyCode::PageDown => scroll::pg_up(window),

                // Build new regex
                KeyCode::Char('/') => {
                    self.clear_matches(window)?;
                    window.redraw()?;
                    window.set_cli_cursor(None)?;
                }

                // Toggle match highlight
                KeyCode::Char('h') => {
                    window.config.highlight_match = !window.config.highlight_match;
                    window.redraw()?;
                }

                // Return to normal
                KeyCode::Esc => self.return_to_normal(window)?,
                _ => {}
            },
            None => match key {
                KeyCode::Enter => {
                    self.set_pattern(window)?;
                    if self.current_pattern.is_some() {
                        self.process_matches(window);
                    };
                    window.redraw()?;
                }
                KeyCode::Esc => self.return_to_normal(window)?,
                key => self.input_handler.recieve_input(window, key)?,
            },
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use regex::bytes::Regex;

    use crate::communication::{
        handlers::{handler::HanderMethods, processor::ProcessorMethods},
        input::input_type::InputType,
        reader::main::MainWindow,
    };

    #[test]
    fn test_can_filter() {
        let mut logria = MainWindow::_new_dummy();
        let mut handler = super::RegexHandler::new();

        // Set state to regex mode
        logria.input_type = InputType::Regex;

        // Set regex pattern
        let pattern = "0";
        handler.current_pattern = Some(Regex::new(pattern).unwrap());
        handler.process_matches(&mut logria);
        assert_eq!(
            vec![0, 10, 20, 30, 40, 50, 60, 70, 80, 90],
            logria.config.matched_rows
        );
    }

    #[test]
    fn test_can_filter_no_matches() {
        let mut logria = MainWindow::_new_dummy();
        let mut handler = super::RegexHandler::new();

        // Set state to regex mode
        logria.input_type = InputType::Regex;

        // Set regex pattern
        let pattern = "a";
        handler.current_pattern = Some(Regex::new(pattern).unwrap());
        logria.config.regex_pattern = Some(Regex::new(pattern).unwrap());
        handler.process_matches(&mut logria);
        assert_eq!(0, logria.config.matched_rows.len());
    }

    #[test]
    fn test_can_return_normal() {
        let mut logria = MainWindow::_new_dummy();
        let mut handler = super::RegexHandler::new();

        // Set state to regex mode
        logria.input_type = InputType::Regex;

        // Set regex pattern
        let pattern = "0";
        handler.current_pattern = Some(Regex::new(pattern).unwrap());
        handler.process_matches(&mut logria);
        handler.return_to_normal(&mut logria).unwrap();

        assert!(handler.current_pattern.is_none());
        assert!(logria.config.regex_pattern.is_none());
        assert_eq!(logria.config.matched_rows.len(), 0);
        assert_eq!(logria.config.last_index_regexed, 0);
    }

    #[test]
    fn test_can_process() {
        let mut logria = MainWindow::_new_dummy();
        let mut handler = super::RegexHandler::new();

        // Set state to regex mode
        logria.input_type = InputType::Regex;

        // Set regex pattern
        let pattern = "0";
        handler.current_pattern = Some(Regex::new(pattern).unwrap());
        handler.process_matches(&mut logria);
        assert_eq!(100, logria.config.last_index_regexed);
    }

    #[test]
    #[should_panic]
    fn test_cant_process_no_pattern() {
        let mut logria = MainWindow::_new_dummy();
        let handler = super::RegexHandler::new();

        // Set state to regex mode
        logria.input_type = InputType::Regex;
        handler.process_matches(&mut logria);
    }

    #[test]
    #[should_panic]
    fn test_test_no_pattern() {
        let mut logria = MainWindow::_new_dummy();
        let handler = super::RegexHandler::new();

        // Set state to regex mode
        logria.input_type = InputType::Regex;
        handler.test("test");
    }
}
