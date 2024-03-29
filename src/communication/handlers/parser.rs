use std::path::Path;

use crossterm::{event::KeyCode, Result};
use regex::Regex;

use crate::{
    communication::{
        handlers::{
            handler::Handler, multiple_choice::MultipleChoiceHandler, processor::ProcessorMethods,
        },
        input::{InputType::Normal, StreamType},
        reader::MainWindow,
    },
    extensions::{
        extension::ExtensionMethods,
        parser::{Parser, PatternType},
    },
    ui::scroll,
    util::error::LogriaError,
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
        &mut self,
        index: usize,
        message: &str,
    ) -> std::result::Result<Option<String>, LogriaError> {
        match self.parser.as_ref().unwrap().pattern_type {
            PatternType::Regex => match self.parser.as_ref().unwrap().get_regex() {
                Ok(pattern) => Ok(self.regex_handle(message, index, pattern)),
                Err(why) => Err(why),
            },
            PatternType::Split => {
                Ok(self.split_handle(message, index, &self.parser.as_ref().unwrap().pattern))
            }
        }
    }

    /// Parse a message with regex logic
    fn regex_handle(&self, message: &str, index: usize, pattern: Regex) -> Option<String> {
        // We add 1 here because the zeroth index of a Capture is the original message
        match pattern.captures(message) {
            Some(caps) => caps
                .get(index.checked_add(1).unwrap_or(index))
                .map(|s| s.as_str().to_owned()),
            None => None,
        }
    }

    /// Parse a message with split logic
    fn split_handle(&self, message: &str, index: usize, pattern: &str) -> Option<String> {
        let result: Vec<&str> = message.split_terminator(pattern).collect();
        result.get(index).map(|part| String::from(*part))
    }

    /// Handle aggregation logic for a single message
    fn aggregate_handle(
        &mut self,
        message: &str,
        num_to_get: &usize,
        render: bool,
    ) -> std::result::Result<Vec<String>, LogriaError> {
        match &mut self.parser {
            Some(parser) => {
                // Split message into a Vec<&str> of its parts
                let message_parts: std::result::Result<Vec<&str>, LogriaError> = match parser
                    .pattern_type
                {
                    PatternType::Regex => match parser.get_regex() {
                        Ok(pattern) => {
                            if let Some(captures) = pattern.captures(message) {
                                Ok(captures
                                    .iter()
                                    .skip(1)
                                    .flatten()
                                    .map(|f| f.as_str())
                                    .collect())
                            } else {
                                Err(LogriaError::CannotParseMessage(
                                    "regex did not match message!".to_string(),
                                ))
                            }
                        }
                        Err(why) => Err(why),
                    },
                    PatternType::Split => Ok(message.split_terminator(&parser.pattern).collect()),
                };

                match message_parts {
                    Ok(message_parts) => {
                        // If we got this far, allocate the return value
                        let mut aggregated_data = vec![];
                        for (idx, part) in message_parts.iter().enumerate() {
                            if let Some(item) = parser.order.get(idx).cloned() {
                                if let Some(aggregator) = parser.aggregator_map.get_mut(&item) {
                                    aggregator.update(part)?;
                                    if render {
                                        // Name of aggregated part
                                        aggregated_data.push(item);
                                        // Messages generated for that aggregator
                                        aggregated_data.extend(aggregator.messages(num_to_get));
                                    }
                                } else {
                                    return Err(LogriaError::InvalidParserState(format!(
                                        "aggregator missing for {}!",
                                        item
                                    )));
                                }
                            } else {
                                return Err(LogriaError::CannotParseMessage(
                                    "number of aggregation methods not equal to number of matches!"
                                        .to_string(),
                                ));
                            }
                        }
                        Ok(aggregated_data)
                    }
                    Err(why) => Err(why),
                }
            }
            None => Err(LogriaError::InvalidParserState(
                "no parser selected!".to_string(),
            )),
        }
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
        self.parser = None;
        window.config.auxiliary_messages.clear();
        window.config.last_index_processed = 0;
        window.config.aggregation_enabled = false;
        self.status.clear();
        window.reset_command_line()?;
        Ok(())
    }

    /// Parse messages, loading the buffer of parsed messages in the main window
    fn process_matches(&mut self, window: &mut MainWindow) -> Result<()> {
        // Only process if the parser is set up properly
        if let ParserState::Full = window.config.parser_state {
            // TODO: Possibly async? Possibly loading indicator for large jobs?
            if self.parser.is_some() {
                // Start from where we left off to the most recent message
                let buf_range = (
                    window.config.last_index_processed,
                    window.previous_messages().len(),
                );

                // Iterate "forever", skipping to the start and taking up till end-start
                // TODO: Something to indicate progress
                let last = buf_range.1.checked_sub(1).unwrap_or(buf_range.0);
                for index in (0..)
                    .skip(buf_range.0)
                    .take(buf_range.1.checked_sub(buf_range.0).unwrap_or(buf_range.0))
                {
                    if window.config.aggregation_enabled {
                        match self.aggregate_handle(
                            &window.previous_messages()[index],
                            &window.config.num_to_aggregate,
                            index == last,
                        ) {
                            Ok(aggregated_messages) => {
                                if !aggregated_messages.is_empty() {
                                    window.config.auxiliary_messages.clear();
                                    window.config.auxiliary_messages.extend(aggregated_messages);
                                }
                            }
                            Err(why) => {
                                // If the message failed parsing, it might just be a different format, so we ignore it
                                // If the parser is in an invalid state, alert the user
                                if let LogriaError::CannotParseMessage(error) = why {
                                    window.write_to_command_line(&error)?;
                                }
                            }
                        }
                    } else if let Ok(Some(message)) = self.parse(
                        window.config.parser_index,
                        &window.previous_messages()[index],
                    ) {
                        window.config.auxiliary_messages.push(message);
                    }
                    // Update the last spot so we know where to start next time
                    window.config.last_index_processed = index + 1;
                }
            }
        };
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

    fn receive_input(&mut self, window: &mut MainWindow, key: KeyCode) -> crossterm::Result<()> {
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
                            let name = Path::new(item).file_name().unwrap().to_str().unwrap();
                            self.status.push_str(&format!("Parsing with {}", name));

                            // Update the parser struct's aggregation map
                            parser.setup();

                            // Set the new parser and parser state
                            self.parser = Some(parser);

                            window.config.parser_state = ParserState::NeedsIndex;

                            // Remove the redraw command for deleted items
                            window.config.generate_auxiliary_messages = None;

                            // Move the cursor back to the start of the line
                            window.go_to_cli()?;

                            // Update the auxillary messages for the second setup step
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
                        self.mc_handler.receive_input(window, key)?;
                    }
                }
            }
            ParserState::NeedsIndex => {
                match self.mc_handler.result {
                    Some(item) => {
                        // Tell the parser to redraw on the next tick
                        self.redraw = true;

                        // get_choice() clears the item from the mc handler)
                        self.mc_handler.get_choice();

                        // Set the new parser index and parser state
                        window.config.parser_index = item;
                        window.config.parser_state = ParserState::Full;

                        // Clear auxillary messages for next use
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
                        self.mc_handler.receive_input(window, key)?;
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
                    KeyCode::PageUp => scroll::pg_up(window),
                    KeyCode::PageDown => scroll::pg_down(window),

                    // Build new parser
                    KeyCode::Char('p') => {
                        // TODO: This does not work
                        self.reset(window);
                    }

                    // Swap to and from analytics mode
                    KeyCode::Char('a') => {
                        if !window.config.aggregation_enabled {
                            let new_status = self.status.to_owned();
                            window.config.current_status = Some(new_status.replace(
                                &format!("field {}", window.config.parser_index),
                                "aggregation mode",
                            ));
                            window.write_status()?;
                            window.config.aggregation_enabled = true;
                        } else {
                            window.config.current_status = Some(self.status.to_owned());
                            window.config.aggregation_enabled = false;
                        }
                        window.config.last_index_processed = 0;
                        window.write_status()?;
                        window.config.auxiliary_messages.clear();
                    }

                    // Return to normal
                    KeyCode::Char('z') | KeyCode::Esc => self.return_to_normal(window)?,

                    _ => {}
                };
            }
        }
        window.redraw()?;
        Ok(())
    }
}

#[cfg(test)]
mod parse_tests {
    use super::ParserHandler;
    use crate::{
        communication::{
            handlers::{handler::Handler, parser::ParserState, processor::ProcessorMethods},
            input::{InputType, StreamType},
            reader::MainWindow,
        },
        extensions::parser::{Parser, PatternType},
        util::aggregators::aggregator::AggregationMethod,
    };
    use std::collections::HashMap;

    #[test]
    fn test_does_split() {
        // Create handler
        let mut handler = ParserHandler::new();

        // Create Parser
        let mut map = HashMap::new();
        map.insert(String::from("1"), AggregationMethod::Count);
        let parser = Parser::new(
            String::from(" - "),
            PatternType::Split,
            String::from("1"),
            vec![String::from("1")],
            map,
        );
        handler.parser = Some(parser);

        let parsed_message = handler.parse(0, "I - Am - A - Test").unwrap().unwrap();

        assert_eq!(parsed_message, String::from("I"))
    }

    #[test]
    fn test_does_regex() {
        // Create handler
        let mut handler = ParserHandler::new();

        // Create Parser
        let mut map = HashMap::new();
        map.insert(String::from("1"), AggregationMethod::Count);
        let parser = Parser::new(
            String::from("(\\d+)"),
            PatternType::Regex,
            String::from("1"),
            vec![String::from("1")],
            map,
        );
        handler.parser = Some(parser);

        let parsed_message = handler
            .parse(0, "Log message part 65 test")
            .unwrap()
            .unwrap();

        assert_eq!(parsed_message, String::from("65"))
    }

    #[test]
    fn test_does_analytics_numbers() {
        // Use the parser sample so we have a second field to look at
        let mut logria = MainWindow::_new_dummy_parse();
        let mut handler = ParserHandler::new();

        // Create Parser
        let mut map = HashMap::new();
        map.insert(String::from("Mean"), AggregationMethod::Mean);
        map.insert(String::from("Sum"), AggregationMethod::Sum);
        map.insert(String::from("Count"), AggregationMethod::Count);
        map.insert(String::from("Mode"), AggregationMethod::Mode);
        let mut parser = Parser::new(
            String::from("([0-9]{0,3}) - ([0-9]{0,3}) - ([0-9]{0,3}) - ([0-9]{0,3})"),
            PatternType::Regex,
            String::from("1 - 2 - 3 - 4"),
            vec![
                String::from("Mean"),
                String::from("Sum"),
                String::from("Count"),
                String::from("Mode"),
            ],
            map,
        );

        parser.setup();

        // Update window config
        handler.parser = Some(parser);
        logria.config.parser_state = ParserState::Full;
        logria.input_type = InputType::Parser;
        logria.config.parser_index = 1;
        logria.config.previous_stream_type = StreamType::StdErr;
        logria.config.aggregation_enabled = true;

        handler.process_matches(&mut logria).unwrap();

        assert_eq!(
            logria.config.auxiliary_messages,
            vec![
                "Mean",
                "    Mean: 59.50",
                "    Count: 100",
                "    Total: 5,950",
                "Sum",
                "    Total: 5,850",
                "Count",
                "    10\u{1b}[0m: 1 (1%)",
                "    100\u{1b}[0m: 1 (1%)",
                "    101\u{1b}[0m: 1 (1%)",
                "    102\u{1b}[0m: 1 (1%)",
                "    103\u{1b}[0m: 1 (1%)",
                "Mode",
                "    10\u{1b}[0m: 1 (1%)",
            ]
        );
    }

    #[test]
    fn test_does_analytics_none() {
        // Use the parser sample so we have a second field to look at
        let mut logria = MainWindow::_new_dummy_parse();
        let mut handler = ParserHandler::new();

        // Create Parser
        let mut map = HashMap::new();
        map.insert(String::from("Mean"), AggregationMethod::None);
        map.insert(String::from("Sum"), AggregationMethod::None);
        map.insert(String::from("Count"), AggregationMethod::None);
        map.insert(String::from("Mode"), AggregationMethod::None);
        let mut parser = Parser::new(
            String::from("([0-9]{0,3}) - ([0-9]{0,3}) - ([0-9]{0,3}) - ([0-9]{0,3})"),
            PatternType::Regex,
            String::from("1 - 2 - 3 - 4"),
            vec![
                String::from("Mean"),
                String::from("Sum"),
                String::from("Count"),
                String::from("Mode"),
            ],
            map,
        );

        parser.setup();

        // Update window config
        handler.parser = Some(parser);
        logria.config.parser_state = ParserState::Full;
        logria.input_type = InputType::Parser;
        logria.config.parser_index = 1;
        logria.config.previous_stream_type = StreamType::StdErr;
        logria.config.aggregation_enabled = true;

        handler.process_matches(&mut logria).unwrap();

        assert_eq!(
            logria.config.auxiliary_messages,
            vec![
                "Mean",
                "    Disabled",
                "Sum",
                "    Disabled",
                "Count",
                "    Disabled",
                "Mode",
                "    Disabled"
            ]
        );
    }

    #[test]
    fn test_does_analytics_dates() {
        // Use the parser sample so we have a second field to look at
        let mut logria = MainWindow::_new_dummy_parse_date();
        let mut handler = ParserHandler::new();

        // Create Parser
        let mut map = HashMap::new();
        map.insert(
            String::from("Date"),
            AggregationMethod::Date("[year]-[month]-[day]".to_string()),
        );
        map.insert(
            String::from("Time"),
            AggregationMethod::Time("[hour]:[minute]:[second]".to_string()),
        );
        map.insert(
            String::from("DateTime"),
            AggregationMethod::DateTime(
                "[year]-[month]-[day] [hour]:[minute]:[second]".to_string(),
            ),
        );
        let mut parser = Parser::new(
            String::from(" | "),
            PatternType::Split,
            String::from("2021-03-19 | 08:10:26 | 2021-03-19 08:10:26"),
            vec![
                String::from("Date"),
                String::from("Time"),
                String::from("DateTime"),
            ],
            map,
        );

        parser.setup();

        // Update window config
        handler.parser = Some(parser);
        logria.config.parser_state = ParserState::Full;
        logria.input_type = InputType::Parser;
        logria.config.parser_index = 1;
        logria.config.previous_stream_type = StreamType::StdErr;
        logria.config.aggregation_enabled = true;

        handler.process_matches(&mut logria).unwrap();

        assert_eq!(
            logria.config.auxiliary_messages,
            vec![
                "Date",
                "    Rate: 4 per week",
                "    Count: 4",
                "    Earliest: 2021-03-10",
                "    Latest: 2021-03-15",
                "Time",
                "    Rate: 4 per minute",
                "    Count: 4",
                "    Earliest: 8:10:26.0",
                "    Latest: 8:10:56.0",
                "DateTime",
                "    Rate: 2 per hour",
                "    Count: 4",
                "    Earliest: 2021-03-19 8:10:26.0",
                "    Latest: 2021-03-19 10:30:26.0"
            ]
        );
    }
}

#[cfg(test)]
mod regex_tests {
    use std::collections::HashMap;

    use crate::{
        communication::{
            handlers::{
                handler::Handler, parser::ParserHandler, parser::ParserState,
                processor::ProcessorMethods,
            },
            input::{InputType, StreamType},
            reader::MainWindow,
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
    fn test_cannot_setup_with_session_invalid_index() {
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
        );

        handler.parser = Some(parser);
        logria.config.parser_state = ParserState::Full;
        logria.input_type = InputType::Parser;
        logria.config.parser_index = 1;
        logria.config.previous_stream_type = StreamType::StdErr;

        handler.process_matches(&mut logria).unwrap();

        assert_eq!(logria.config.auxiliary_messages, Vec::<String>::new());
    }

    #[test]
    fn test_can_setup_with_session_second_index() {
        // Use the parser sample so we have a second field to look at
        let mut logria = MainWindow::_new_dummy_parse();
        let mut handler = ParserHandler::new();

        // Create Parser
        let mut map = HashMap::new();
        map.insert(String::from("1"), AggregationMethod::Count);
        map.insert(String::from("2"), AggregationMethod::Count);
        map.insert(String::from("3"), AggregationMethod::Count);
        map.insert(String::from("4"), AggregationMethod::Count);
        let parser = Parser::new(
            String::from("([1-9]{0,2}) - ([1-9]{0,2}) - ([1-9]{0,2}) - ([1-9]{0,2})"),
            PatternType::Regex,
            String::from("1 - 2 - 3 - 4"),
            vec![String::from("1")],
            map,
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
            vec!["9", "12", "13", "14", "15", "16", "17", "18", "19", "22"]
        );
    }

    #[test]
    fn test_can_setup_with_session_aggregated() {
        let mut logria = MainWindow::_new_dummy_parse();
        // Add some messages that can be easily parsed
        let mut handler = ParserHandler::new();

        // Create Parser
        let mut map = HashMap::new();
        map.insert(String::from("full"), AggregationMethod::Mean);
        map.insert(String::from("minus_1"), AggregationMethod::Mean);
        map.insert(String::from("minus_2"), AggregationMethod::Mean);
        map.insert(String::from("minus_3"), AggregationMethod::Mean);
        let mut parser = Parser::new(
            String::from("(\\d*?) - (\\d*?) - (\\d*?) - (\\d*?)$"),
            PatternType::Regex,
            String::from("1 - 2 - 3 - 4"),
            vec![
                String::from("full"),
                String::from("minus_1"),
                String::from("minus_2"),
                String::from("minus_3"),
            ],
            map,
        );

        parser.setup();

        // Update window config
        handler.parser = Some(parser);
        logria.config.parser_state = ParserState::Full;
        logria.input_type = InputType::Parser;
        logria.config.parser_index = 1;
        logria.config.previous_stream_type = StreamType::StdErr;
        logria.config.aggregation_enabled = true;

        handler.process_matches(&mut logria).unwrap();
        assert_eq!(
            logria.config.auxiliary_messages,
            vec![
                "full",
                "    Mean: 59.50",
                "    Count: 100",
                "    Total: 5,950",
                "minus_1",
                "    Mean: 58.50",
                "    Count: 100",
                "    Total: 5,850",
                "minus_2",
                "    Mean: 57.50",
                "    Count: 100",
                "    Total: 5,750",
                "minus_3",
                "    Mean: 56.50",
                "    Count: 100",
                "    Total: 5,650"
            ]
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
            input::{InputType, StreamType},
            reader::MainWindow,
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
        let mut logria = MainWindow::_new_dummy_parse();
        // Add some messages that can be easily parsed
        let mut handler = ParserHandler::new();

        // Create Parser
        let mut map = HashMap::new();
        map.insert(String::from("full"), AggregationMethod::Mean);
        map.insert(String::from("minus_1"), AggregationMethod::Mean);
        map.insert(String::from("minus_2"), AggregationMethod::Mean);
        map.insert(String::from("minus_3"), AggregationMethod::Mean);
        let mut parser = Parser::new(
            String::from(" - "),
            PatternType::Split,
            String::from("1"),
            vec![
                String::from("full"),
                String::from("minus_1"),
                String::from("minus_2"),
                String::from("minus_3"),
            ],
            map,
        );

        parser.setup();

        // Update window config
        handler.parser = Some(parser);
        logria.config.parser_state = ParserState::Full;
        logria.input_type = InputType::Parser;
        logria.config.parser_index = 1;
        logria.config.previous_stream_type = StreamType::StdErr;
        logria.config.aggregation_enabled = true;

        handler.process_matches(&mut logria).unwrap();
        assert_eq!(
            logria.config.auxiliary_messages,
            vec![
                "full",
                "    Mean: 59.50",
                "    Count: 100",
                "    Total: 5,950",
                "minus_1",
                "    Mean: 58.50",
                "    Count: 100",
                "    Total: 5,850",
                "minus_2",
                "    Mean: 57.50",
                "    Count: 100",
                "    Total: 5,750",
                "minus_3",
                "    Mean: 56.50",
                "    Count: 100",
                "    Total: 5,650"
            ]
        );
    }
}

#[cfg(test)]
mod failure_tests {
    use super::ParserHandler;
    use std::collections::HashMap;

    use crate::{
        communication::{
            handlers::{handler::Handler, parser::ParserState, processor::ProcessorMethods},
            input::{InputType, StreamType},
            reader::MainWindow,
        },
        extensions::parser::{Parser, PatternType},
        util::aggregators::aggregator::AggregationMethod,
    };

    #[test]
    fn test_no_matches_for_order() {
        let mut logria = MainWindow::_new_dummy_parse();
        // Add some messages that can be easily parsed
        let mut handler = ParserHandler::new();

        // Create Parser
        let mut map = HashMap::new();
        map.insert(String::from("full"), AggregationMethod::Mean);
        map.insert(String::from("minus_1"), AggregationMethod::Mean);
        map.insert(String::from("minus_2"), AggregationMethod::Mean);
        map.insert(String::from("minus_3"), AggregationMethod::Mean);

        let mut parser = Parser::new(
            String::from(" - "),
            PatternType::Split,
            String::from("1"),
            vec![
                String::from("full"),
                String::from("minus_1"),
                String::from("minus_2"),
            ],
            map,
        );

        parser.setup();

        // Update window config
        handler.parser = Some(parser);
        logria.config.parser_state = ParserState::Full;
        logria.input_type = InputType::Parser;
        logria.config.parser_index = 1;
        logria.config.previous_stream_type = StreamType::StdErr;
        logria.config.aggregation_enabled = true;

        handler.process_matches(&mut logria).unwrap();
        assert_eq!(logria.config.auxiliary_messages, Vec::<String>::new());
    }

    #[test]
    fn test_unbalanced_aggregation_methods() {
        let mut logria = MainWindow::_new_dummy_parse();
        // Add some messages that can be easily parsed
        let mut handler = ParserHandler::new();

        // Create Parser
        let mut map = HashMap::new();
        map.insert(String::from("full"), AggregationMethod::Mean);
        map.insert(String::from("minus_1"), AggregationMethod::Mean);
        map.insert(String::from("minus_2"), AggregationMethod::Mean);
        let mut parser = Parser::new(
            String::from(" - "),
            PatternType::Split,
            String::from("1"),
            vec![
                String::from("full"),
                String::from("minus_1"),
                String::from("minus_2"),
                String::from("minus_3"),
            ],
            map,
        );

        // Only 3 aggregators, not enough for 4 matches
        parser.setup();

        // Update window config
        handler.parser = Some(parser);
        logria.config.parser_state = ParserState::Full;
        logria.input_type = InputType::Parser;
        logria.config.parser_index = 1;
        logria.config.previous_stream_type = StreamType::StdErr;
        logria.config.aggregation_enabled = true;

        handler.process_matches(&mut logria).unwrap();
        assert_eq!(logria.config.auxiliary_messages, Vec::<String>::new());
    }
}
