use crossterm::{event::KeyCode, Result};
use regex::bytes::Regex;

use super::{handler::Handler, processor::ProcessorMethods};
use crate::{
    communication::{
        handlers::user_input::UserInputHandler, input::InputType::Normal, reader::MainWindow,
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

    /// Save the user input pattern to the main window config
    fn set_pattern(&mut self, window: &mut MainWindow) -> Result<()> {
        let pattern = match self.input_handler.gather(window) {
            Ok(pattern) => pattern,
            Err(why) => panic!("Unable to gather text: {:?}", why),
        };

        self.current_pattern = match Regex::new(&pattern) {
            Ok(regex) => {
                window.config.current_status = Some(format!("Regex with pattern /{}/", pattern));
                window.write_status()?;

                // Update the main window's regex
                window.config.regex_pattern = Some(regex.to_owned());
                Some(regex)
            }
            Err(e) => {
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
    fn process_matches(&mut self, window: &mut MainWindow) -> Result<()> {
        // TODO: Possibly async? Possibly loading indicator for large jobs?
        if self.current_pattern.is_some() {
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
        Ok(())
    }

    /// Return the app to a normal input state
    fn return_to_normal(&mut self, window: &mut MainWindow) -> Result<()> {
        self.clear_matches(window)?;
        window.config.current_status = None;
        window.update_input_type(Normal)?;
        window.set_cli_cursor(None)?;
        self.input_handler.gather(window)?;
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

impl Handler for RegexHandler {
    fn new() -> RegexHandler {
        RegexHandler {
            color_pattern: Regex::new(ANSI_COLOR_PATTERN).unwrap(),
            current_pattern: None,
            input_handler: UserInputHandler::new(),
        }
    }

    fn receive_input(&mut self, window: &mut MainWindow, key: KeyCode) -> Result<()> {
        match &self.current_pattern {
            Some(_) => match key {
                // Scroll
                KeyCode::Down => scroll::down(window),
                KeyCode::Up => scroll::up(window),
                KeyCode::Left => scroll::top(window),
                KeyCode::Right => scroll::bottom(window),
                KeyCode::Home => scroll::top(window),
                KeyCode::End => scroll::bottom(window),
                KeyCode::PageUp => scroll::pg_up(window),
                KeyCode::PageDown => scroll::pg_down(window),

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

                // Enter command mode
                KeyCode::Char(':') => window.set_command_mode(None)?,

                // Return to normal
                KeyCode::Esc => self.return_to_normal(window)?,
                _ => {}
            },
            None => match key {
                KeyCode::Enter => {
                    self.set_pattern(window)?;
                    if self.current_pattern.is_some() {
                        window.reset_output()?;
                        self.process_matches(window)?;
                    };
                    window.redraw()?;
                }
                KeyCode::Esc => self.return_to_normal(window)?,
                key => self.input_handler.receive_input(window, key)?,
            },
        }
        window.redraw()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::KeyCode;
    use regex::bytes::Regex;

    use crate::communication::{
        handlers::{handler::Handler, processor::ProcessorMethods},
        input::InputType,
        reader::MainWindow,
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
        handler.process_matches(&mut logria).unwrap();
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
        handler.process_matches(&mut logria).unwrap();
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
        handler.process_matches(&mut logria).unwrap();
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
        handler.process_matches(&mut logria).unwrap();
        assert_eq!(100, logria.config.last_index_regexed);
    }

    #[test]
    fn test_can_process_no_pattern() {
        let mut logria = MainWindow::_new_dummy();
        let mut handler = super::RegexHandler::new();

        // Set state to regex mode
        logria.input_type = InputType::Regex;
        handler.process_matches(&mut logria).unwrap();

        assert_eq!(logria.config.matched_rows, Vec::<usize>::new());
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

    #[test]
    fn test_can_enter_command_mode() {
        let mut logria = MainWindow::_new_dummy();
        let mut handler = super::RegexHandler::new();

        // Set state to regex mode
        logria.input_type = InputType::Regex;

        // Set regex pattern
        let pattern = "0";
        handler.current_pattern = Some(Regex::new(pattern).unwrap());

        // Normally this is set by `set_pattern()` but that requires user input
        logria.config.regex_pattern = Some(Regex::new(pattern).unwrap());
        handler.process_matches(&mut logria).unwrap();

        // Simulate keystroke for command mode
        handler
            .receive_input(&mut logria, KeyCode::Char(':'))
            .unwrap();

        // Ensure we have the same amount of messages as when the regex was active
        assert_eq!(
            logria.config.matched_rows.len(),
            logria.number_of_messages()
        );
    }
}
