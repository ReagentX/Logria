use crossterm::{event::KeyCode, Result};
use regex::Regex;

use super::{handler::HanderMethods, processor::ProcessorMethods};
use crate::{
    communication::{
        handlers::user_input::UserInputHandler, input::input_type::InputType::Normal,
        reader::main::MainWindow,
    },
    extensions::parser::{Parser, PatternType},
    ui::scroll,
};

pub struct ParserHandler {
    input_handler: UserInputHandler,
}

impl ParserHandler {
    /// Setup the parser instance on the main window
    fn setup_parser(&self, window: &mut MainWindow) {
        // TODO: Make this work
        window.config.parser = Some(Parser::load("fake_name"));
    }

    /// Parse a message with the current parser rules
    fn parse(&self, parser: &Parser, index: usize, message: &str) -> Option<String> {
        match parser.pattern_type {
            PatternType::Regex => match Regex::new(&parser.pattern) {
                Ok(pattern) => self.regex_handle(message, index, pattern),
                Err(_) => None,
            },
            PatternType::Split => self.split_handle(message, index, &parser.pattern),
        }
    }

    /// Parse message with regex logic
    fn regex_handle(&self, message: &str, index: usize, pattern: Regex) -> Option<String> {
        if let Some(captures) = pattern.captures(message) {
            Some(captures.get(index).unwrap().as_str().to_owned())
        } else {
            None
        }
    }

    /// Parse message with split logic
    fn split_handle(&self, message: &str, index: usize, pattern: &str) -> Option<String> {
        let result: Vec<&str> = message.split_terminator(pattern).collect();
        match result.get(index) {
            Some(part) => Some(String::from(*part)),
            None => None,
        }
    }
}

impl ProcessorMethods for ParserHandler {
    /// Return the app to a normal input state
    fn return_to_normal(&mut self, window: &mut MainWindow) -> Result<()> {
        self.clear_matches(window)?;
        window.input_type = Normal;
        window.set_cli_cursor(None)?;
        window.redraw()?;
        Ok(())
    }

    /// Clear the parsed messages from the message buffer
    fn clear_matches(&mut self, window: &mut MainWindow) -> Result<()> {
        // TODO: Determine if regex while parsing still works after parser deactivation
        window.config.parser = None;
        window.config.parsed_messages.clear();
        window.config.last_index_processed = 0;
        window.reset_command_line()?;
        Ok(())
    }

    /// Parse messages, loading the buffer of parsed messages in the main window
    fn process_matches(&self, window: &mut MainWindow) {
        // TODO: Possibly async? Possibly loading indicator for large jobs?
        match &window.config.parser {
            Some(parser) => {
                // Start from where we left off to the most recent message
                let buf_range = (window.config.last_index_processed, window.messages().len());

                // Iterate "forever", skipping to the start and taking up till end-start
                // TODO: Something to indicate progress
                for index in (0..).skip(buf_range.0).take(buf_range.1 - buf_range.0) {
                    if let Some(message) = self.parse(
                        parser,
                        window.config.parser_index,
                        &window.messages()[index],
                    ) {
                        window.config.parsed_messages.push(message);
                    }

                    // Update the last spot so we know where to start next time
                    window.config.last_index_processed = index + 1;
                }
            }
            None => {
                self.setup_parser(window);
            }
        }
    }
}

impl HanderMethods for ParserHandler {
    fn new() -> ParserHandler {
        ParserHandler {
            input_handler: UserInputHandler::new(),
        }
    }

    fn recieve_input(&mut self, window: &mut MainWindow, key: KeyCode) -> Result<()> {
        window.write_to_command_line("got data in ParserHandler")?;
        match key {
            // Scroll
            KeyCode::Down => scroll::down(window),
            KeyCode::Up => scroll::up(window),
            KeyCode::Left => scroll::top(window),
            KeyCode::Right => scroll::bottom(window),
            KeyCode::Home => scroll::top(window),
            KeyCode::End => scroll::bottom(window),
            KeyCode::PageUp => scroll::pg_down(window),
            KeyCode::PageDown => scroll::pg_up(window),

            // Build new parser
            KeyCode::Char('p') => {
                self.setup_parser(window);
            }

            KeyCode::Char('z') => {
                self.return_to_normal(window)?;
            }

            // Return to normal
            KeyCode::Esc => self.return_to_normal(window)?,
            key => self.input_handler.recieve_input(window, key)?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod regex_tests {
    use super::ParserHandler;

    use crate::{
        communication::{
            handlers::{handler::HanderMethods, processor::ProcessorMethods},
            input::input_type::InputType,
            reader::main::MainWindow,
        },
        extensions::parser::Parser,
    };

    #[test]
    fn test_can_setup_with_session_first_index() {
        // Update window config
        let mut logria = MainWindow::_new_dummy();
        let handler = ParserHandler::new();
        let parser = Parser::load("Digit Test");

        logria.config.parser = Some(parser);
        logria.input_type = InputType::Parser;
        logria.config.parser_index = 0;

        handler.process_matches(&mut logria);

        assert_eq!(
            logria.config.parsed_messages[0..10],
            vec!["1", "2", "3", "4", "5", "6", "7", "8", "9", "1"]
        );
    }

    #[test]
    fn test_can_setup_with_session_second_index() {
        let mut logria = MainWindow::_new_dummy();
        let handler = ParserHandler::new();
        let parser = Parser::load("Digit Test");

        // Update window config
        logria.config.parser = Some(parser);
        logria.input_type = InputType::Parser;
        logria.config.parser_index = 1;

        handler.process_matches(&mut logria);

        assert_eq!(
            logria.config.parsed_messages[0..10],
            vec!["1", "2", "3", "4", "5", "6", "7", "8", "9", "1"]
        );
    }
}

#[cfg(test)]
mod split_tests {
    use super::ParserHandler;

    use crate::{
        communication::{
            handlers::{handler::HanderMethods, processor::ProcessorMethods},
            input::input_type::InputType,
            reader::main::MainWindow,
        },
        extensions::parser::Parser,
    };

    #[test]
    fn test_can_setup_with_session_first_index() {
        // Update window config
        let mut logria = MainWindow::_new_dummy();
        let handler = ParserHandler::new();
        let parser = Parser::load("Char Test");

        logria.config.parser = Some(parser);
        logria.input_type = InputType::Parser;
        logria.config.parser_index = 0;

        handler.process_matches(&mut logria);
        assert_eq!(
            logria.config.parsed_messages[0..10],
            vec!["0", "", "2", "3", "4", "5", "6", "7", "8", "9"]
        );
        assert_eq!(
            logria.config.parsed_messages[15..25],
            vec!["", "", "", "", "", "20", "2", "22", "23", "24"]
        );
    }

    #[test]
    fn test_can_setup_with_session_second_index() {
        let mut logria = MainWindow::_new_dummy();
        let handler = ParserHandler::new();
        let parser = Parser::load("Char Test");

        // Update window config
        logria.config.parser = Some(parser);
        logria.input_type = InputType::Parser;
        logria.config.parser_index = 1;

        handler.process_matches(&mut logria);
        assert_eq!(
            logria.config.parsed_messages[0..10],
            vec!["0", "", "2", "3", "4", "5", "6", "7", "8", "9"]
        );
        assert_eq!(logria.config.parsed_messages.len(), 10)
    }
}
