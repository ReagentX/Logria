use crossterm::{event::KeyCode, Result};
use regex::Regex;

use crate::{
    communication::{
        handlers::{
            handler::Handler, multiple_choice::MultipleChoiceHandler, processor::ProcessorMethods,
        },
        input::{input_type::InputType::Normal, stream_type::StreamType},
        reader::main::MainWindow,
    },
    extensions::{
        extension::ExtensionMethods,
        parser::{Parser, PatternType},
    },
    ui::scroll,
    util::{
        aggregators::{
            aggregator::AggregationMethod,
            counter::Counter,
            date::{Date, DateParserType},
            mean::Mean,
            sum::Sum,
        },
        error::LogriaError,
    },
};

#[derive(Debug, PartialEq)]
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
    parser: Option<Parser>,
}

impl ParserHandler {
    /// Setup the parser instance on the main window
    // TODO: Make this and select_index send proper function handle to window.config.generate_auxiliary_messages
    // TODO: Pretty sure the above is done, need to double check
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
        if let Some(parser) = &self.parser {
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
        aggregate: &bool,
    ) -> std::result::Result<Option<String>, LogriaError> {
        if *aggregate {
            // Perform analytics here
            Ok(self.aggregate_handle(parser, message))
        } else {
            match parser.pattern_type {
                PatternType::Regex => match parser.get_regex() {
                    Ok(pattern) => Ok(self.regex_handle(message, index, pattern)),
                    Err(why) => Err(why),
                },
                PatternType::Split => Ok(self.split_handle(message, index, &parser.pattern)),
            }
        }
    }

    /// Parse a message with regex logic
    fn regex_handle(&self, message: &str, index: usize, pattern: Regex) -> Option<String> {
        pattern
            .captures(message)
            .map(|captures| captures.get(index).unwrap().as_str().to_owned())
    }

    /// Parse a message with split logic
    fn split_handle(&self, message: &str, index: usize, pattern: &str) -> Option<String> {
        let result: Vec<&str> = message.split_terminator(pattern).collect();
        result.get(index).map(|part| String::from(*part))
    }

    /// Handle aggregation logic for a single message
    fn aggregate_handle(&self, parser: &Parser, message: &str) -> Option<String> {
        // Split message into a Vec<&str> of its parts
        let message_parts: Vec<&str> = match parser.pattern_type {
            PatternType::Regex => match parser.get_regex() {
                Ok(pattern) => Ok(pattern
                    .captures(message)
                    .unwrap()
                    .iter()
                    .flatten()
                    .map(|f| f.as_str())
                    .collect()),
                Err(why) => Err(why),
            },
            PatternType::Split => Ok(message.split_terminator(&parser.pattern).collect()),
        }
        .unwrap_or_default();
        for (idx, part) in message_parts.iter().enumerate() {
            let item = parser.order.get(idx).unwrap();
            let method = parser.aggregation_methods.get(item).unwrap();
            if parser.aggregator_map.contains_key(item) {
                todo!();
            }
            todo!()
        }
        Some(String::from(message))
    }

    /// Reset parser
    fn reset(&mut self, window: &mut MainWindow) {
        // Parser still active, but not set up
        window.config.parser_state = ParserState::NeedsParser;
        window.config.auxiliary_messages.clear();
        self.parser = None;
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
        window.config.current_status = None;
        window.redraw()?;
        Ok(())
    }

    /// Clear the parsed messages from the message buffer
    fn clear_matches(&mut self, window: &mut MainWindow) -> Result<()> {
        // TODO: Determine if regex while parsing still works after parser deactivation
        self.parser = None;
        window.config.auxiliary_messages.clear();
        window.config.last_index_processed = 0;
        self.status.clear();
        window.reset_command_line()?;
        Ok(())
    }

    /// Parse messages, loading the buffer of parsed messages in the main window
    fn process_matches(&mut self, window: &mut MainWindow) -> Result<()> {
        // Only process if the parser is set up properly
        if let ParserState::Full = window.config.parser_state {
            // TODO: Possibly async? Possibly loading indicator for large jobs?
            match &self.parser {
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
                            &window.config.aggregation_enabled,
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

impl Handler for ParserHandler {
    fn new() -> ParserHandler {
        ParserHandler {
            mc_handler: MultipleChoiceHandler::new(),
            redraw: true,
            status: String::new(),
            parser: None,
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
                        Ok(mut parser) => {
                            // Tell the parser to redraw on the next tick
                            self.redraw = true;

                            // Update the status string
                            self.status.push_str(&format!("Parsing with {}", item));

                            // Update the parser struct's aggregation map
                            for method_name in &parser.order {
                                if let Some(method) = parser.aggregation_methods.get(method_name) {
                                    match method {
                                        AggregationMethod::Mean => {
                                            parser
                                                .aggregator_map
                                                .insert(item.to_string(), Box::new(Mean::new()));
                                        }
                                        AggregationMethod::Mode => {
                                            parser
                                                .aggregator_map
                                                .insert(item.to_string(), Box::new(Counter::new()));
                                        }
                                        AggregationMethod::Sum => {
                                            parser
                                                .aggregator_map
                                                .insert(item.to_string(), Box::new(Sum::new()));
                                        }
                                        AggregationMethod::Count => {
                                            parser
                                                .aggregator_map
                                                .insert(item.to_string(), Box::new(Counter::new()));
                                        }
                                        AggregationMethod::Date(format) => {
                                            parser.aggregator_map.insert(
                                                item.to_string(),
                                                Box::new(Date::new(format, DateParserType::Date)),
                                            );
                                        }
                                        AggregationMethod::Time(format) => {
                                            parser.aggregator_map.insert(
                                                item.to_string(),
                                                Box::new(Date::new(format, DateParserType::Time)),
                                            );
                                        }
                                        AggregationMethod::DateTime(format) => {
                                            parser.aggregator_map.insert(
                                                item.to_string(),
                                                Box::new(Date::new(
                                                    format,
                                                    DateParserType::DateTime,
                                                )),
                                            );
                                        }
                                    };
                                }
                            }

                            // Set the new parser and parser state
                            self.parser = Some(parser);

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
                            window.redraw()?;
                        }
                        window.render_auxiliary_text()?;
                        self.select_parser(window)?;
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
                        window.config.current_status = Some(self.status.to_owned());
                        window.write_status()?;
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

                    // Swap to and from analytics mode
                    KeyCode::Char('a') => {
                        window.config.aggregation_enabled = !window.config.aggregation_enabled;
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
mod parse_tests {
    use super::ParserHandler;
    use crate::{
        communication::handlers::handler::Handler,
        extensions::parser::{Parser, PatternType},
        util::aggregators::aggregator::AggregationMethod,
    };
    use std::collections::HashMap;

    #[test]
    fn test_does_split() {
        // Create handler
        let handler = ParserHandler::new();

        // Create Parser
        let mut map = HashMap::new();
        map.insert(String::from("1"), AggregationMethod::Count);
        let parser = Parser::new(
            String::from(" - "),
            PatternType::Split,
            String::from("1"),
            vec![String::from("1")],
            map,
            None,
        );

        let parsed_message = handler
            .parse(&parser, 0, "I - Am - A - Test", &false)
            .unwrap()
            .unwrap();

        assert_eq!(parsed_message, String::from("I"))
    }

    #[test]
    fn test_does_regex() {
        // Create handler
        let handler = ParserHandler::new();

        // Create Parser
        let mut map = HashMap::new();
        map.insert(String::from("1"), AggregationMethod::Count);
        let parser = Parser::new(
            String::from("(\\d+)"),
            PatternType::Regex,
            String::from("1"),
            vec![String::from("1")],
            map,
            None,
        );

        let parsed_message = handler
            .parse(&parser, 0, "Log message part 65 test", &false)
            .unwrap()
            .unwrap();

        assert_eq!(parsed_message, String::from("65"))
    }

    #[test]
    fn test_does_analytics_average() {
        // TODO: Implement tests for every parser method
    }
}

#[cfg(test)]
mod regex_tests {
    use std::collections::HashMap;

    use super::ParserHandler;

    use crate::{
        communication::{
            handlers::{handler::Handler, parser::ParserState, processor::ProcessorMethods},
            input::{input_type::InputType, stream_type::StreamType},
            reader::main::MainWindow,
        },
        extensions::parser::{Parser, PatternType},
        util::aggregators::aggregator::AggregationMethod,
    };

    #[test]
    fn test_can_setup_with_session_first_index() {
        // Update window config
        let mut logria = MainWindow::_new_dummy();
        let mut handler = ParserHandler::new();

        // Create Parser
        let mut map = HashMap::new();
        map.insert(String::from("1"), AggregationMethod::Count);
        let parser = Parser::new(
            String::from("([1-9])"),
            PatternType::Regex,
            String::from("1"),
            vec![String::from("1")],
            map,
            None,
        );

        handler.parser = Some(parser);
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
        let mut handler = ParserHandler::new();

        // Create Parser
        let mut map = HashMap::new();
        map.insert(String::from("1"), AggregationMethod::Count);
        let parser = Parser::new(
            String::from("([1-9])"),
            PatternType::Regex,
            String::from("1"),
            vec![String::from("1")],
            map,
            None,
        );

        // Update window config
        handler.parser = Some(parser);
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

    #[test]
    fn test_can_setup_with_session_aggregated() {
        let mut logria = MainWindow::_new_dummy();
        let mut handler = ParserHandler::new();

        // Create Parser
        let mut map = HashMap::new();
        // TODO: Build working test case and parser
        map.insert(String::from("1"), AggregationMethod::Mean);
        let parser = Parser::new(
            String::from("([1-9])"),
            PatternType::Regex,
            String::from("1"),
            vec![String::from("1")],
            map,
            None,
        );

        // Update window config
        handler.parser = Some(parser);
        logria.config.parser_state = ParserState::Full;
        logria.input_type = InputType::Parser;
        logria.config.parser_index = 1;
        logria.config.previous_stream_type = StreamType::StdErr;
        logria.config.aggregation_enabled = true;

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
            handlers::{handler::Handler, parser::ParserState, processor::ProcessorMethods},
            input::{input_type::InputType, stream_type::StreamType},
            reader::main::MainWindow,
        },
        extensions::parser::{Parser, PatternType},
        util::aggregators::aggregator::AggregationMethod,
    };

    #[test]
    fn test_can_setup_with_session_first_index() {
        // Update window config
        let mut logria = MainWindow::_new_dummy();
        let mut handler = ParserHandler::new();

        // Create Parser
        let mut map = HashMap::new();
        map.insert(String::from("1"), AggregationMethod::Count);
        let parser = Parser::new(
            String::from("1"),
            PatternType::Split,
            String::from("1"),
            vec![String::from("1")],
            map,
            None,
        );

        handler.parser = Some(parser);
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
        let mut handler = ParserHandler::new();

        // Create Parser
        let mut map = HashMap::new();
        map.insert(String::from("1"), AggregationMethod::Count);
        let parser = Parser::new(
            String::from("1"),
            PatternType::Split,
            String::from("1"),
            vec![String::from("1")],
            map,
            None,
        );

        // Update window config
        handler.parser = Some(parser);
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

    #[test]
    fn test_can_setup_with_session_aggregated() {
        let mut logria = MainWindow::_new_dummy();
        // Add some messages that can be easily parsed
        let mut handler = ParserHandler::new();

        // Create Parser
        let mut map = HashMap::new();
        map.insert(String::from("1"), AggregationMethod::Mean);
        let parser = Parser::new(
            String::from("-"),
            PatternType::Split,
            String::from("1"),
            vec![String::from("1")],
            map,
            None,
        );

        // Update window config
        handler.parser = Some(parser);
        logria.config.parser_state = ParserState::Full;
        logria.input_type = InputType::Parser;
        logria.config.parser_index = 1;
        logria.config.previous_stream_type = StreamType::StdErr;
        logria.config.aggregation_enabled = true;

        handler.process_matches(&mut logria).unwrap();
        assert_eq!(
            logria.config.auxiliary_messages[0..10],
            vec!["0", "", "2", "3", "4", "5", "6", "7", "8", "9"]
        );
        assert_eq!(logria.config.auxiliary_messages.len(), 10)
    }
}
