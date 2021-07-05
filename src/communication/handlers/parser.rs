use crossterm::{event::KeyCode, Result};
use regex::Regex;

use crate::{
    communication::{
        handlers::{
            handler::HanderMethods, multiple_choice::MultipleChoiceHandler,
            processor::ProcessorMethods,
        },
        input::{input_type::InputType::Normal, stream_type::StreamType},
        reader::main::MainWindow,
    },
    extensions::{
        extension::ExtensionMethods,
        parser::{Parser, PatternType},
    },
    ui::scroll,
    util::error::LogriaError,
};

#[derive(Debug)]
pub enum ParserState {
    Disabled,
    NeedsParser,
    NeedsIndex,
    Full,
}

pub struct ParserHandler {
    mc_handler: MultipleChoiceHandler,
    redraw: bool,   // True if we should redraw the choices in the window
    status: String, // Stores the current parser and index for the user
}

impl ParserHandler {
    /// Setup the parser instance on the main window
    // TODO: Make this and select_index send proper function handle to window.config.generate_auxiliary_messages
    // So that we render the text when it updates from deletion commands
    pub fn parser_messages_handle() -> Vec<String> {
        let mut body_text = vec![];
        Parser::list_clean()
            .iter()
            .enumerate()
            .for_each(|(index, choice)| body_text.push(format!("{}: {}", index, choice)));
        body_text
    }

    fn select_parser(&mut self, window: &mut MainWindow) -> Result<()> {
        let parsers = Parser::list_full();
        self.mc_handler.set_choices(&parsers);
        window.render_auxiliary_text()?;
        Ok(())
    }

    /// Set which index of the parsed message to render
    fn select_index(&mut self, window: &mut MainWindow) -> Result<()> {
        if let Some(parser) = &window.config.parser {
            match parser.get_example() {
                Ok(examples) => {
                    self.mc_handler.set_choices(&examples);
                }
                Err(why) => {
                    window.write_to_command_line(&why.to_string())?;
                }
            }
        }
        window.config.auxiliary_messages.clear();
        window
            .config
            .auxiliary_messages
            .extend(self.mc_handler.get_body_text());
        window.redraw()?;
        Ok(())
    }

    /// Parse a message with the current parser rules
    fn parse(
        &self,
        parser: &Parser,
        index: usize,
        message: &str,
    ) -> std::result::Result<Option<String>, LogriaError> {
        match parser.pattern_type {
            PatternType::Regex => match parser.get_regex() {
                Ok(pattern) => Ok(self.regex_handle(message, index, pattern)),
                Err(why) => Err(why),
            },
            PatternType::Split => Ok(self.split_handle(message, index, &parser.pattern)),
        }
    }

    /// Parse message with regex logic
    fn regex_handle(&self, message: &str, index: usize, pattern: Regex) -> Option<String> {
        pattern
            .captures(message)
            .map(|captures| captures.get(index).unwrap().as_str().to_owned())
    }

    /// Parse message with split logic
    fn split_handle(&self, message: &str, index: usize, pattern: &str) -> Option<String> {
        let result: Vec<&str> = message.split_terminator(pattern).collect();
        result.get(index).map(|part| String::from(*part))
    }

    /// Reset parser
    fn reset(&self, window: &mut MainWindow) {
        // Parser still active, but not set up
        window.config.parser_state = ParserState::NeedsParser;
        window.config.auxiliary_messages.clear();
        window.config.parser = None;
        window.config.parser_index = 0;
        window.config.did_switch = true;
    }
}

impl ProcessorMethods for ParserHandler {
    /// Return the app to a normal input state
    fn return_to_normal(&mut self, window: &mut MainWindow) -> Result<()> {
        self.clear_matches(window)?;
        self.redraw = true;
        window.update_input_type(Normal)?;
        window.set_cli_cursor(None)?;
        window.config.stream_type = window.config.previous_stream_type;
        window.config.parser_state = ParserState::Disabled;
        window.redraw()?;
        Ok(())
    }

    /// Clear the parsed messages from the message buffer
    fn clear_matches(&mut self, window: &mut MainWindow) -> Result<()> {
        // TODO: Determine if regex while parsing still works after parser deactivation
        window.config.parser = None;
        window.config.auxiliary_messages.clear();
        window.config.last_index_processed = 0;
        self.status.clear();
        window.reset_command_line()?;
        Ok(())
    }

    /// Parse messages, loading the buffer of parsed messages in the main window
    fn process_matches(&self, window: &mut MainWindow) -> Result<()> {
        // Only process if the parser is set up properly
        if let ParserState::Full = window.config.parser_state {
            // TODO: Possibly async? Possibly loading indicator for large jobs?
            match &window.config.parser {
                Some(parser) => {
                    // Start from where we left off to the most recent message
                    let buf_range = (
                        window.config.last_index_processed,
                        window.previous_messages().len(),
                    );

                    // Iterate "forever", skipping to the start and taking up till end-start
                    // TODO: Something to indicate progress
                    // TODO: Overflow subtraction
                    for index in (0..).skip(buf_range.0).take(buf_range.1 - buf_range.0) {
                        if let Ok(Some(message)) = self.parse(
                            parser,
                            window.config.parser_index,
                            &window.previous_messages()[index],
                        ) {
                            window.config.auxiliary_messages.push(message);
                        }

                        // Update the last spot so we know where to start next time
                        window.config.last_index_processed = index + 1;
                    }
                }
                None => {
                    panic!("Parser state is Full but there is no Parser!");
                }
            };
        }
        Ok(())
    }
}

impl HanderMethods for ParserHandler {
    fn new() -> ParserHandler {
        ParserHandler {
            mc_handler: MultipleChoiceHandler::new(),
            redraw: true,
            status: String::new(),
        }
    }

    fn recieve_input(&mut self, window: &mut MainWindow, key: KeyCode) -> crossterm::Result<()> {
        // Enable command mode for parsers
        if key == KeyCode::Char(':') {
            window.set_command_mode(Some(Parser::del))?;
            // Early escape to not send a `:` char to the rest of this method
            return Ok(());
        }

        // Handle special cases for setup
        match window.config.parser_state {
            ParserState::Disabled | ParserState::NeedsParser => {
                match self.mc_handler.get_choice() {
                    Some(item) => match Parser::load(item) {
                        Ok(parser) => {
                            // Tell the parser to redraw on the next tick
                            self.redraw = true;

                            // Update the status string
                            self.status
                                .push_str(&format!("Parsing with {}", item));

                            // Set the new parser and parser state
                            window.config.parser = Some(parser);
                            window.config.parser_state = ParserState::NeedsIndex;

                            // Remove the redraw command for deleted items
                            window.config.generate_auxiliary_messages = None;

                            // Move the cursor back to the start of the line
                            window.go_to_cli()?;

                            // Update the auxilery messages for the second setup step
                            self.select_index(window)?;
                        }
                        Err(why) => {
                            window.write_to_command_line(&why.to_string())?;
                        }
                    },
                    None => {
                        if self.redraw {
                            // First loop this case hits
                            window.config.stream_type = StreamType::Auxiliary;
                            window.config.parser_state = ParserState::NeedsParser;
                            window.config.generate_auxiliary_messages =
                                Some(ParserHandler::parser_messages_handle);
                            self.redraw = false;
                            self.select_parser(window)?;
                            window.redraw()?;
                        }
                        self.mc_handler.recieve_input(window, key)?;
                    }
                }
            }
            ParserState::NeedsIndex => {
                match self.mc_handler.result {
                    Some(item) => {
                        // Tell the parser to redraw on the next tick
                        self.redraw = true;
                        // TODO: More graceful clearing of the mc handler value
                        // get_choice() clears the item from the mc handler)
                        self.mc_handler.get_choice();

                        // Set the new parser index and parser state
                        window.config.parser_index = item;
                        window.config.parser_state = ParserState::Full;

                        // Clear auxilery messages for next use
                        window.config.auxiliary_messages.clear();

                        // Process messages
                        self.process_matches(window)?;

                        // Update the status string
                        self.status.push_str(&format!(", field {}", item));

                        // Clear the screen for new messages
                        window.reset_output()?;

                        // Write the new parser status to the command line
                        window.write_to_command_line(&self.status)?;
                    }
                    None => {
                        self.mc_handler.recieve_input(window, key)?;
                    }
                }
            }
            ParserState::Full => {
                // Handle user input selection
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
                        self.reset(window);
                    }

                    // Return to normal
                    KeyCode::Char('z') | KeyCode::Esc => self.return_to_normal(window)?,

                    _ => {}
                };
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod regex_tests {
    use std::collections::HashMap;

    use super::ParserHandler;

    use crate::{
        communication::{
            handlers::{handler::HanderMethods, parser::ParserState, processor::ProcessorMethods},
            input::{input_type::InputType, stream_type::StreamType},
            reader::main::MainWindow,
        },
        extensions::parser::{Parser, PatternType},
    };

    #[test]
    fn test_can_setup_with_session_first_index() {
        // Update window config
        let mut logria = MainWindow::_new_dummy();
        let handler = ParserHandler::new();

        // Create Parser
        let mut map = HashMap::new();
        map.insert(String::from("1"), String::from("count"));
        let parser = Parser::new(
            String::from("([1-9])"),
            PatternType::Regex,
            String::from("1"),
            map,
            None,
        );

        logria.config.parser = Some(parser);
        logria.config.parser_state = ParserState::Full;
        logria.input_type = InputType::Parser;
        logria.config.parser_index = 0;
        logria.config.previous_stream_type = StreamType::StdErr;

        handler.process_matches(&mut logria).unwrap();

        assert_eq!(
            logria.config.auxiliary_messages[0..10],
            vec!["1", "2", "3", "4", "5", "6", "7", "8", "9", "1"]
        );
    }

    #[test]
    fn test_can_setup_with_session_second_index() {
        let mut logria = MainWindow::_new_dummy();
        let handler = ParserHandler::new();

        // Create Parser
        let mut map = HashMap::new();
        map.insert(String::from("1"), String::from("count"));
        let parser = Parser::new(
            String::from("([1-9])"),
            PatternType::Regex,
            String::from("1"),
            map,
            None,
        );

        // Update window config
        logria.config.parser = Some(parser);
        logria.config.parser_state = ParserState::Full;
        logria.input_type = InputType::Parser;
        logria.config.parser_index = 1;
        logria.config.previous_stream_type = StreamType::StdErr;

        handler.process_matches(&mut logria).unwrap();

        assert_eq!(
            logria.config.auxiliary_messages[0..10],
            vec!["1", "2", "3", "4", "5", "6", "7", "8", "9", "1"]
        );
    }
}

#[cfg(test)]
mod split_tests {
    use super::ParserHandler;
    use std::collections::HashMap;

    use crate::{
        communication::{
            handlers::{handler::HanderMethods, parser::ParserState, processor::ProcessorMethods},
            input::{input_type::InputType, stream_type::StreamType},
            reader::main::MainWindow,
        },
        extensions::parser::{Parser, PatternType},
    };

    #[test]
    fn test_can_setup_with_session_first_index() {
        // Update window config
        let mut logria = MainWindow::_new_dummy();
        let handler = ParserHandler::new();

        // Create Parser
        let mut map = HashMap::new();
        map.insert(String::from("1"), String::from("count"));
        let parser = Parser::new(
            String::from("1"),
            PatternType::Split,
            String::from("1"),
            map,
            None,
        );

        logria.config.parser = Some(parser);
        logria.config.parser_state = ParserState::Full;
        logria.input_type = InputType::Parser;
        logria.config.parser_index = 0;
        logria.config.previous_stream_type = StreamType::StdErr;

        handler.process_matches(&mut logria).unwrap();
        assert_eq!(
            logria.config.auxiliary_messages[0..10],
            vec!["0", "", "2", "3", "4", "5", "6", "7", "8", "9"]
        );
        assert_eq!(
            logria.config.auxiliary_messages[15..25],
            vec!["", "", "", "", "", "20", "2", "22", "23", "24"]
        );
    }

    #[test]
    fn test_can_setup_with_session_second_index() {
        let mut logria = MainWindow::_new_dummy();
        let handler = ParserHandler::new();

        // Create Parser
        let mut map = HashMap::new();
        map.insert(String::from("1"), String::from("count"));
        let parser = Parser::new(
            String::from("1"),
            PatternType::Split,
            String::from("1"),
            map,
            None,
        );

        // Update window config
        logria.config.parser = Some(parser);
        logria.config.parser_state = ParserState::Full;
        logria.input_type = InputType::Parser;
        logria.config.parser_index = 1;
        logria.config.previous_stream_type = StreamType::StdErr;

        handler.process_matches(&mut logria).unwrap();
        assert_eq!(
            logria.config.auxiliary_messages[0..10],
            vec!["0", "", "2", "3", "4", "5", "6", "7", "8", "9"]
        );
        assert_eq!(logria.config.auxiliary_messages.len(), 10)
    }
}
