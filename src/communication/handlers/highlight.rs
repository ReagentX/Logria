use std::io::Result;

use crossterm::event::KeyCode;
use regex::bytes::Regex;

use super::{handler::Handler, processor::ProcessorMethods};
use crate::{
    communication::{
        handlers::user_input::UserInputHandler, input::InputType::Normal, reader::MainWindow,
    },
    constants::cli::{
        cli_chars::{COMMAND_CHAR, HIGHLIGHT_CHAR, NORMAL_STR, TOGGLE_HIGHLIGHT_CHAR},
        patterns::ANSI_COLOR_PATTERN,
    },
    ui::scroll::{self, update_current_match_index, ScrollState},
};

pub struct HighlightHandler {
    color_pattern: Regex,
    current_pattern: Option<Regex>,
    input_handler: UserInputHandler,
}

impl HighlightHandler {
    /// Test a message to see if it matches the pattern while also escaping the color code
    fn test(&self, message: &str) -> bool {
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
            Err(why) => panic!("Unable to gather text: {why:?}"),
        };

        self.current_pattern = match Regex::new(&pattern) {
            Ok(regex) => {
                window.config.current_status = Some(format!("Highlight with pattern /{pattern}/"));
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

    /// Internal implementation of pg_up to skip to the previous match
    fn pg_up(&self, window: &mut MainWindow) {
        update_current_match_index(window, true);
    }

    /// Internal implementation of pg_down to skip to the next match
    fn pg_down(&self, window: &mut MainWindow) {
        update_current_match_index(window, false);
    }
}

impl ProcessorMethods for HighlightHandler {
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
        // Handle reset of scroll state
        window.config.current_matched_row = 0;
        if matches!(window.config.scroll_state, ScrollState::Centered) {
            window.config.scroll_state = ScrollState::Free;
        }

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
        window.config.scroll_state = ScrollState::Free;
        window.reset_command_line()?;
        Ok(())
    }
}

impl Handler for HighlightHandler {
    fn new() -> HighlightHandler {
        HighlightHandler {
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
                KeyCode::PageUp => self.pg_up(window),
                KeyCode::PageDown => self.pg_down(window),

                // Build new regex
                KeyCode::Char(HIGHLIGHT_CHAR) => {
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
                    self.set_pattern(window)?;
                    if self.current_pattern.is_some() {
                        window.reset_output()?;
                        self.process_matches(window)?;
                    }
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

    use crate::{
        communication::{
            handlers::{
                handler::Handler, highlight::HighlightHandler, processor::ProcessorMethods,
            },
            input::InputType,
            reader::MainWindow,
        },
        constants::cli::cli_chars::COMMAND_CHAR,
        ui::scroll::ScrollState,
    };

    #[test]
    fn test_can_filter() {
        let mut logria = MainWindow::_new_dummy();
        let mut handler = HighlightHandler::new();

        // Set state to highlight mode
        logria.input_type = InputType::Highlight;

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
        let mut handler = HighlightHandler::new();

        // Set state to highlight mode
        logria.input_type = InputType::Highlight;

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
        let mut handler = HighlightHandler::new();

        // Set state to highlight mode
        logria.input_type = InputType::Highlight;

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
        let mut handler = HighlightHandler::new();

        // Set state to highlight mode
        logria.input_type = InputType::Highlight;

        // Set regex pattern
        let pattern = "0";
        handler.current_pattern = Some(Regex::new(pattern).unwrap());
        handler.process_matches(&mut logria).unwrap();
        assert_eq!(100, logria.config.last_index_regexed);
    }

    #[test]
    fn test_can_process_no_pattern() {
        let mut logria = MainWindow::_new_dummy();
        let mut handler = HighlightHandler::new();

        // Set state to highlight mode
        logria.input_type = InputType::Highlight;
        handler.process_matches(&mut logria).unwrap();

        assert_eq!(logria.config.matched_rows, Vec::<usize>::new());
    }

    #[test]
    #[should_panic]
    fn test_test_no_pattern() {
        let mut logria = MainWindow::_new_dummy();
        let handler = HighlightHandler::new();

        // Set state to highlight mode
        logria.input_type = InputType::Highlight;
        handler.test("test");
    }

    #[test]
    fn test_can_enter_command_mode() {
        let mut logria = MainWindow::_new_dummy();
        let mut handler = HighlightHandler::new();

        // Set state to highlight mode
        logria.input_type = InputType::Highlight;

        // Set regex pattern
        let pattern = "0";
        handler.current_pattern = Some(Regex::new(pattern).unwrap());

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

    #[test]
    fn test_can_scroll_to_prev_match_top() {
        let mut logria = MainWindow::_new_dummy();
        let mut handler = HighlightHandler::new();

        // Set state to highlight mode
        logria.input_type = InputType::Highlight;
        logria.config.scroll_state = ScrollState::Top;

        // Set regex pattern
        let pattern = "0"; // Matches every 10th message
        handler.current_pattern = Some(Regex::new(pattern).unwrap());
        handler.process_matches(&mut logria).unwrap();

        // Simulate keystroke for page up
        handler.receive_input(&mut logria, KeyCode::PageUp).unwrap();

        assert_eq!(logria.config.current_matched_row, 0);
        assert_eq!(
            logria.config.matched_rows[logria.config.current_matched_row],
            0
        );
        assert_eq!(
            logria.messages()[logria.config.matched_rows[logria.config.current_matched_row]],
            "0"
        );

        // Simulate keystroke for page up
        handler.receive_input(&mut logria, KeyCode::PageUp).unwrap();

        assert_eq!(logria.config.current_matched_row, 0);
        assert_eq!(
            logria.config.matched_rows[logria.config.current_matched_row],
            0
        );
        assert_eq!(
            logria.messages()[logria.config.matched_rows[logria.config.current_matched_row]],
            "0"
        );
    }

    #[test]
    fn test_can_scroll_to_prev_match_bottom() {
        let mut logria = MainWindow::_new_dummy();
        let mut handler = HighlightHandler::new();

        // Set state to highlight mode
        logria.input_type = InputType::Highlight;
        logria.config.scroll_state = ScrollState::Bottom;

        // Set regex pattern
        let pattern = "0"; // Matches every 10th message
        handler.current_pattern = Some(Regex::new(pattern).unwrap());
        handler.process_matches(&mut logria).unwrap();

        // Simulate keystroke for page up
        handler.receive_input(&mut logria, KeyCode::PageUp).unwrap();

        assert_eq!(logria.config.current_matched_row, 9);
        assert_eq!(
            logria.config.matched_rows[logria.config.current_matched_row],
            90
        );
        assert_eq!(
            logria.messages()[logria.config.matched_rows[logria.config.current_matched_row]],
            "90"
        );
    }

    #[test]
    fn test_can_scroll_to_prev_match_free() {
        let mut logria = MainWindow::_new_dummy();
        let mut handler = HighlightHandler::new();

        // Set state to highlight mode
        logria.input_type = InputType::Highlight;
        logria.config.scroll_state = ScrollState::Bottom;
        logria.config.previous_render = (50, 57);
        logria.config.current_end = 57;

        // Set regex pattern
        let pattern = "0"; // Matches every 10th message
        handler.current_pattern = Some(Regex::new(pattern).unwrap());
        handler.process_matches(&mut logria).unwrap();

        // Simulate keystroke for scroll up to change to free scrolling, then page up
        handler.receive_input(&mut logria, KeyCode::Up).unwrap();
        handler.receive_input(&mut logria, KeyCode::PageUp).unwrap();

        assert_eq!(logria.config.current_matched_row, 5);
        assert_eq!(
            logria.config.matched_rows[logria.config.current_matched_row],
            50
        );
        assert_eq!(
            logria.messages()[logria.config.matched_rows[logria.config.current_matched_row]],
            "50"
        );

        // Simulate keystroke for page up
        handler.receive_input(&mut logria, KeyCode::PageUp).unwrap();

        assert_eq!(logria.config.current_matched_row, 4);
        assert_eq!(
            logria.config.matched_rows[logria.config.current_matched_row],
            40
        );
        assert_eq!(
            logria.messages()[logria.config.matched_rows[logria.config.current_matched_row]],
            "40"
        );
    }

    #[test]
    fn test_can_scroll_to_prev_match_centered() {
        let mut logria = MainWindow::_new_dummy();
        let mut handler = HighlightHandler::new();

        // Set state to highlight mode
        logria.input_type = InputType::Highlight;
        logria.config.scroll_state = ScrollState::Free;
        logria.config.previous_render = (50, 57);
        logria.config.current_end = 57;

        // Set regex pattern
        let pattern = "0"; // Matches every 10th message
        handler.current_pattern = Some(Regex::new(pattern).unwrap());
        handler.process_matches(&mut logria).unwrap();

        // Simulate keystroke for scroll up, then page up
        handler.receive_input(&mut logria, KeyCode::Up).unwrap();
        handler.receive_input(&mut logria, KeyCode::PageUp).unwrap();

        assert_eq!(logria.config.current_matched_row, 5);
        assert_eq!(
            logria.config.matched_rows[logria.config.current_matched_row],
            50
        );
        assert_eq!(
            logria.messages()[logria.config.matched_rows[logria.config.current_matched_row]],
            "50"
        );

        // Simulate keystroke for page up
        handler.receive_input(&mut logria, KeyCode::PageUp).unwrap();

        assert_eq!(logria.config.current_matched_row, 4);
        assert_eq!(
            logria.config.matched_rows[logria.config.current_matched_row],
            40
        );
        assert_eq!(
            logria.messages()[logria.config.matched_rows[logria.config.current_matched_row]],
            "40"
        );
    }

    #[test]
    fn test_can_scroll_to_next_match_top() {
        let mut logria = MainWindow::_new_dummy();
        let mut handler = HighlightHandler::new();

        // Set state to highlight mode
        logria.input_type = InputType::Highlight;
        logria.config.scroll_state = ScrollState::Top;

        // Set regex pattern
        let pattern = "0"; // Matches every 10th message
        handler.current_pattern = Some(Regex::new(pattern).unwrap());
        handler.process_matches(&mut logria).unwrap();

        // Simulate keystroke for page down
        handler
            .receive_input(&mut logria, KeyCode::PageDown)
            .unwrap();

        assert_eq!(logria.config.current_matched_row, 0);
        assert_eq!(
            logria.config.matched_rows[logria.config.current_matched_row],
            0
        );
        assert_eq!(
            logria.messages()[logria.config.matched_rows[logria.config.current_matched_row]],
            "0"
        );

        // Simulate keystroke for page down
        handler
            .receive_input(&mut logria, KeyCode::PageDown)
            .unwrap();

        assert_eq!(logria.config.current_matched_row, 1);
        assert_eq!(
            logria.config.matched_rows[logria.config.current_matched_row],
            10
        );
        assert_eq!(
            logria.messages()[logria.config.matched_rows[logria.config.current_matched_row]],
            "10"
        );
    }

    #[test]
    fn test_can_scroll_to_next_match_bottom() {
        let mut logria = MainWindow::_new_dummy();
        let mut handler = HighlightHandler::new();

        // Set state to highlight mode
        logria.input_type = InputType::Highlight;
        logria.config.scroll_state = ScrollState::Bottom;

        // Set regex pattern
        let pattern = "0"; // Matches every 10th message
        handler.current_pattern = Some(Regex::new(pattern).unwrap());
        handler.process_matches(&mut logria).unwrap();

        // Simulate keystroke for page down
        handler
            .receive_input(&mut logria, KeyCode::PageDown)
            .unwrap();

        assert_eq!(logria.config.current_matched_row, 9);
        assert_eq!(
            logria.config.matched_rows[logria.config.current_matched_row],
            90
        );
        assert_eq!(
            logria.messages()[logria.config.matched_rows[logria.config.current_matched_row]],
            "90"
        );

        // Simulate keystroke for page down
        handler
            .receive_input(&mut logria, KeyCode::PageDown)
            .unwrap();

        assert_eq!(logria.config.current_matched_row, 9);
        assert_eq!(
            logria.config.matched_rows[logria.config.current_matched_row],
            90
        );
        assert_eq!(
            logria.messages()[logria.config.matched_rows[logria.config.current_matched_row]],
            "90"
        );
    }

    #[test]
    fn test_can_scroll_to_next_match_free() {
        let mut logria = MainWindow::_new_dummy();
        let mut handler = HighlightHandler::new();

        // Set state to highlight mode
        logria.input_type = InputType::Highlight;
        logria.config.scroll_state = ScrollState::Bottom;
        logria.config.previous_render = (50, 57);
        logria.config.current_end = 57;

        // Set regex pattern
        let pattern = "0"; // Matches every 10th message
        handler.current_pattern = Some(Regex::new(pattern).unwrap());
        handler.process_matches(&mut logria).unwrap();

        // Simulate keystroke for scroll up to change to free scrolling, then page down
        handler.receive_input(&mut logria, KeyCode::Up).unwrap();
        handler
            .receive_input(&mut logria, KeyCode::PageDown)
            .unwrap();

        assert_eq!(logria.config.current_matched_row, 5);
        assert_eq!(
            logria.config.matched_rows[logria.config.current_matched_row],
            50
        );
        assert_eq!(
            logria.messages()[logria.config.matched_rows[logria.config.current_matched_row]],
            "50"
        );

        // Simulate keystroke for page down
        handler
            .receive_input(&mut logria, KeyCode::PageDown)
            .unwrap();

        assert_eq!(logria.config.current_matched_row, 6);
        assert_eq!(
            logria.config.matched_rows[logria.config.current_matched_row],
            60
        );
        assert_eq!(
            logria.messages()[logria.config.matched_rows[logria.config.current_matched_row]],
            "60"
        );
    }

    #[test]
    fn test_can_scroll_to_next_match_centered() {
        let mut logria = MainWindow::_new_dummy();
        let mut handler = HighlightHandler::new();

        // Set state to highlight mode
        logria.input_type = InputType::Highlight;
        logria.config.scroll_state = ScrollState::Free;
        logria.config.previous_render = (50, 57);
        logria.config.current_end = 57;

        // Set regex pattern
        let pattern = "0"; // Matches every 10th message
        handler.current_pattern = Some(Regex::new(pattern).unwrap());
        handler.process_matches(&mut logria).unwrap();

        // Simulate keystroke for scroll up, then page up
        handler.receive_input(&mut logria, KeyCode::Up).unwrap();
        handler
            .receive_input(&mut logria, KeyCode::PageDown)
            .unwrap();

        assert_eq!(logria.config.current_matched_row, 5);
        assert_eq!(
            logria.config.matched_rows[logria.config.current_matched_row],
            50
        );
        assert_eq!(
            logria.messages()[logria.config.matched_rows[logria.config.current_matched_row]],
            "50"
        );

        // Simulate keystroke for page up
        handler
            .receive_input(&mut logria, KeyCode::PageDown)
            .unwrap();

        assert_eq!(logria.config.current_matched_row, 6);
        assert_eq!(
            logria.config.matched_rows[logria.config.current_matched_row],
            60
        );
        assert_eq!(
            logria.messages()[logria.config.matched_rows[logria.config.current_matched_row]],
            "60"
        );
    }

    #[test]
    fn test_scroll_snap_to_midpoint() {
        let mut logria = MainWindow::_new_dummy();
        let mut handler = HighlightHandler::new();

        // Set state to highlight mode
        logria.input_type = InputType::Highlight;
        logria.config.scroll_state = ScrollState::Free;
        logria.config.previous_render = (50, 57);
        logria.config.current_end = 57;

        // Set regex pattern
        let pattern = "0"; // Matches every 10th message
        handler.current_pattern = Some(Regex::new(pattern).unwrap());
        handler.process_matches(&mut logria).unwrap();

        // Simulate keystroke for scroll up, then page up
        handler.receive_input(&mut logria, KeyCode::Up).unwrap();
        handler.receive_input(&mut logria, KeyCode::PageUp).unwrap();

        assert_eq!(logria.config.current_matched_row, 5);
        assert_eq!(
            logria.config.matched_rows[logria.config.current_matched_row],
            50
        );
        assert_eq!(
            logria.messages()[logria.config.matched_rows[logria.config.current_matched_row]],
            "50"
        );

        // Simulate keystroke for scroll down, then page down
        handler.receive_input(&mut logria, KeyCode::Down).unwrap();
        handler
            .receive_input(&mut logria, KeyCode::PageDown)
            .unwrap();

        assert_eq!(logria.config.current_matched_row, 5);
        assert_eq!(
            logria.config.matched_rows[logria.config.current_matched_row],
            50
        );
        assert_eq!(
            logria.messages()[logria.config.matched_rows[logria.config.current_matched_row]],
            "50"
        );
    }
}
