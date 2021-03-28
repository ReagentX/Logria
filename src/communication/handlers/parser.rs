use std::{result, usize};

use crossterm::{event::KeyCode, Result};
use regex::{Error, Regex};

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
    /// Setup the parser
    fn setup_parser(&self, window: &mut MainWindow) {
        // TODO: Make this work
        window.config.parser = Some(Parser::load("fake_name"));
    }

    /// Parse a message with the current parser rules
    fn parse(&self, parser: &Parser, index: usize, message: &str) -> result::Result<String, Error> {
        match parser.pattern_type {
            PatternType::Regex => match Regex::new(&parser.pattern) {
                Ok(pattern) => Ok(self.regex_handle(message, index, pattern)),
                Err(why) => Err(why),
            },
            PatternType::Split => Ok(self.split_handle(message, index, &parser.pattern)),
        }
    }

    /// Parse message with regex logic
    fn regex_handle(&self, message: &str, index: usize, pattern: Regex) -> String {
        let result: Vec<&str> = pattern.split(message).collect();
        result[index].to_owned()
    }

    /// Parse message with split logic
    fn split_handle(&self, message: &str, index: usize, pattern: &str) -> String {
        let result: Vec<&str> = message.split(pattern).collect();
        result[index].to_owned()
    }
}

impl ProcessorMethods for ParserHandler {
    fn return_to_normal(&mut self, window: &mut MainWindow) -> Result<()> {
        self.clear_matches(window)?;
        window.input_type = Normal;
        window.set_cli_cursor(None)?;
        window.redraw()?;
        Ok(())
    }

    fn clear_matches(&mut self, window: &mut MainWindow) -> Result<()> {
        window.config.parser = None;
        window.config.regex_pattern = None;
        window.config.parsed_messages.clear();
        window.config.last_index_processed = 0;
        window.config.highlight_match = false;
        window.reset_command_line()?;
        Ok(())
    }

    /// Process matches, loading the buffer of indexes to matched messages in the main buffer
    fn process_matches(&self, window: &mut MainWindow) {
        // TODO: Possibly async? Possibly loading indicator for large jobs?
        match &window.config.parser {
            Some(parser) => {
                // Start from where we left off to the most recent message
                let buf_range = (window.config.last_index_processed, window.messages().len());

                // Iterate "forever", skipping to the start and taking up till end-start
                // TODO: Something to indicate progress
                for index in (0..).skip(buf_range.0).take(buf_range.1 - buf_range.0) {
                    match self.parse(
                        parser,
                        window.config.parser_index,
                        &window.messages()[index],
                    ) {
                        Ok(message) => {
                            window.config.parsed_messages.push(message);
                        }
                        Err(why) => {
                            window
                                .config
                                .parsed_messages
                                .push(format!("Unable to parse message: {:?}", why));
                            break;
                        }
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

            // Return to normal
            KeyCode::Esc => self.return_to_normal(window)?,
            key => self.input_handler.recieve_input(window, key)?,
        }
        Ok(())
    }
}
