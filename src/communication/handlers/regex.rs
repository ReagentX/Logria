use std::io::Result;

use crossterm::event::KeyCode;

use super::{handler::Handler, processor::ProcessorMethods, search::SearchState};
use crate::{
    communication::reader::MainWindow,
    constants::cli::cli_chars::{COMMAND_CHAR, REGEX_CHAR, TOGGLE_HIGHLIGHT_CHAR},
    ui::scroll,
};

pub struct RegexHandler {
    search: SearchState,
}

impl ProcessorMethods for RegexHandler {
    fn process_matches(&mut self, window: &mut MainWindow) -> Result<()> {
        self.search.process_matches(window)
    }

    fn return_to_normal(&mut self, window: &mut MainWindow) -> Result<()> {
        self.search.return_to_normal(window)
    }

    fn clear_matches(&mut self, window: &mut MainWindow) -> Result<()> {
        self.search.clear_matches(window)
    }
}

impl Handler for RegexHandler {
    fn new() -> RegexHandler {
        RegexHandler {
            search: SearchState::new(),
        }
    }

    fn receive_input(&mut self, window: &mut MainWindow, key: KeyCode) -> Result<()> {
        match &self.search.current_pattern {
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
                KeyCode::Char(REGEX_CHAR) => {
                    self.clear_matches(window)?;
                    window.redraw()?;
                    window.set_cli_cursor(None)?;
                }

                // Toggle match highlight
                KeyCode::Char(TOGGLE_HIGHLIGHT_CHAR) => {
                    window.config.highlight_match = !window.config.highlight_match;
                    window.redraw()?;
                }

                // Enter command mode
                KeyCode::Char(COMMAND_CHAR) => window.set_command_mode(None)?,

                // Return to normal
                KeyCode::Esc => self.return_to_normal(window)?,
                _ => {}
            },
            None => match key {
                KeyCode::Enter => {
                    self.search.set_pattern(window, "Regex")?;
                    if self.search.current_pattern.is_some() {
                        window.reset_output()?;
                        self.process_matches(window)?;
                    }
                    window.redraw()?;
                }
                KeyCode::Esc => self.return_to_normal(window)?,
                key => self.search.input_handler.receive_input(window, key)?,
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

    use crate::{
        communication::{
            handlers::{handler::Handler, processor::ProcessorMethods, regex::RegexHandler},
            input::InputType,
            reader::MainWindow,
        },
        constants::cli::cli_chars::COMMAND_CHAR,
    };

    #[test]
    fn test_can_filter() {
        let mut logria = MainWindow::_new_dummy();
        let mut handler = RegexHandler::new();

        // Set state to regex mode
        logria.input_type = InputType::Regex;

        // Set regex pattern
        let pattern = "0";
        handler.search.current_pattern = Some(Regex::new(pattern).unwrap());
        handler.process_matches(&mut logria).unwrap();
        assert_eq!(
            vec![0, 10, 20, 30, 40, 50, 60, 70, 80, 90],
            logria.config.matched_rows
        );
    }

    #[test]
    fn test_can_filter_no_matches() {
        let mut logria = MainWindow::_new_dummy();
        let mut handler = RegexHandler::new();

        // Set state to regex mode
        logria.input_type = InputType::Regex;

        // Set regex pattern
        let pattern = "a";
        handler.search.current_pattern = Some(Regex::new(pattern).unwrap());
        logria.config.regex_pattern = Some(Regex::new(pattern).unwrap());
        handler.process_matches(&mut logria).unwrap();
        assert_eq!(0, logria.config.matched_rows.len());
    }

    #[test]
    fn test_can_return_normal() {
        let mut logria = MainWindow::_new_dummy();
        let mut handler = RegexHandler::new();

        // Set state to regex mode
        logria.input_type = InputType::Regex;

        // Set regex pattern
        let pattern = "0";
        handler.search.current_pattern = Some(Regex::new(pattern).unwrap());
        handler.process_matches(&mut logria).unwrap();
        handler.return_to_normal(&mut logria).unwrap();

        assert!(handler.search.current_pattern.is_none());
        assert!(logria.config.regex_pattern.is_none());
        assert_eq!(logria.config.matched_rows.len(), 0);
        assert_eq!(logria.config.last_index_regexed, 0);
    }

    #[test]
    fn test_can_process() {
        let mut logria = MainWindow::_new_dummy();
        let mut handler = RegexHandler::new();

        // Set state to regex mode
        logria.input_type = InputType::Regex;

        // Set regex pattern
        let pattern = "0";
        handler.search.current_pattern = Some(Regex::new(pattern).unwrap());
        handler.process_matches(&mut logria).unwrap();
        assert_eq!(100, logria.config.last_index_regexed);
    }

    #[test]
    fn test_can_process_no_pattern() {
        let mut logria = MainWindow::_new_dummy();
        let mut handler = RegexHandler::new();

        // Set state to regex mode
        logria.input_type = InputType::Regex;
        handler.process_matches(&mut logria).unwrap();

        assert_eq!(logria.config.matched_rows, Vec::<usize>::new());
    }

    #[test]
    #[should_panic]
    fn test_test_no_pattern() {
        let mut logria = MainWindow::_new_dummy();
        let handler = RegexHandler::new();

        // Set state to regex mode
        logria.input_type = InputType::Regex;
        handler.search.test("test");
    }

    #[test]
    fn test_can_enter_command_mode() {
        let mut logria = MainWindow::_new_dummy();
        let mut handler = RegexHandler::new();

        // Set state to regex mode
        logria.input_type = InputType::Regex;

        // Set regex pattern
        let pattern = "0";
        handler.search.current_pattern = Some(Regex::new(pattern).unwrap());

        // Normally this is set by `set_pattern()` but that requires user input
        logria.config.regex_pattern = Some(Regex::new(pattern).unwrap());
        handler.process_matches(&mut logria).unwrap();

        // Simulate keystroke for command mode
        handler
            .receive_input(&mut logria, KeyCode::Char(COMMAND_CHAR))
            .unwrap();

        // Ensure we have the same amount of messages as when the regex was active
        assert_eq!(logria.config.matched_rows.len(), 10);

        // Ensure we are in command mode
        assert_eq!(logria.input_type, InputType::Command);
    }
}
