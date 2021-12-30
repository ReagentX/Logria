use std::io::Write;

use crossterm::{event::KeyCode, Result};

use super::handler::Handler;
use crate::{
    communication::{
        handlers::user_input::UserInputHandler,
        input::{input_type::InputType, stream_type::StreamType},
        reader::main::MainWindow,
    },
    ui::scroll::ScrollState,
    util::{credits::gen, error::LogriaError},
};

pub struct CommandHandler {
    input_handler: UserInputHandler,
}

impl CommandHandler {
    fn return_to_prev_state(&mut self, window: &mut MainWindow) -> Result<()> {
        // If we are in auxiliary mode, go back to that, otherwise go to normal mode
        window.update_input_type(window.previous_input_type)?;
        window.write_status()?;
        window.config.delete_func = None;
        window.set_cli_cursor(None)?;
        window.output.flush()?;
        Ok(())
    }

    fn resolve_poll_rate(&self, command: &str) -> std::result::Result<u64, LogriaError> {
        let parts: Vec<&str> = command.split(' ').collect(); // ["poll", "42", ...]
        if parts.len() < 2 {
            return Err(LogriaError::InvalidCommand(format!(
                "No poll delay provided {:?}",
                parts
            )));
        }
        match parts[1].parse::<u64>() {
            Ok(parsed) => Ok(parsed),
            Err(why) => Err(LogriaError::InvalidCommand(format!("{:?}", why))),
        }
    }

    fn resolve_aggregation_count(&self, command: &str) -> std::result::Result<usize, LogriaError> {
        let parts: Vec<&str> = command.split(' ').collect(); // ["agg", "42", ...]
        if parts.len() < 2 {
            return Err(LogriaError::InvalidCommand(format!(
                "No aggregation count provided: {:?}",
                parts
            )));
        }
        match parts[1].parse::<usize>() {
            Ok(parsed) => Ok(parsed),
            Err(why) => Err(LogriaError::InvalidCommand(format!("{:?}", why))),
        }
    }

    fn resolve_delete_command(
        &self,
        command: &str,
    ) -> std::result::Result<Vec<usize>, LogriaError> {
        // Validate length
        if command.len() < 3 {
            return Err(LogriaError::InvalidCommand(format!("{:?}", command)));
        }

        // Remove "r " from the string
        let parts = command[2..].split(',');
        let mut out_l: Vec<usize> = vec![];

        // Not for_each because we may need to bail early
        for part in parts {
            if part.contains('-') {
                // Create range
                let range: Vec<&str> = part.split('-').collect();
                if range.len() != 2 {
                    continue;
                }

                // Parse range
                // This code is repeated because we cannot break from the loop if we use a closure
                match (range[0].parse::<usize>(), range[1].parse::<usize>()) {
                    (Ok(start), Ok(end)) => {
                        (start..end + 1).for_each(|step| out_l.push(step));
                    }
                    (_, _) => {
                        return Err(LogriaError::InvalidCommand(format!(
                            "range invalid: {:?}",
                            &range
                        )))
                    }
                }

                // Add all items to the range
            } else {
                // Parse the value
                if !part.is_empty() {
                    match part.parse::<usize>() {
                        Ok(num) => {
                            out_l.push(num);
                        }
                        Err(why) => return Err(LogriaError::InvalidCommand(format!("{:?}", why))),
                    }
                }
            }
        }
        out_l.sort_unstable();
        Ok(out_l)
    }

    fn process_command(&mut self, window: &mut MainWindow, command: &str) -> Result<()> {
        if command == "q" {
            window.quit()?;
        }
        // Update poll rate
        else if command.starts_with("poll ") {
            match self.resolve_poll_rate(command) {
                Ok(val) => {
                    window.config.poll_rate = val;
                }
                Err(why) => {
                    window.write_to_command_line(&format!(
                        "Failed to parse remove command: {:?}",
                        why
                    ))?;
                }
            }
        }
        // Enter configuration mode
        else if command.starts_with("config") {
            // TODO: Make this work
            window.write_to_command_line("Config mode")?
        }
        // Enter history mode
        else if command.starts_with("history") {
            // TODO: Make this work
            window.write_to_command_line("History mode")?
        }
        // Exit history mode
        else if command.starts_with("history off") {
            // TODO: Make this work
            window.write_to_command_line("History off")?
        }
        // Go back to start screen, must be before `: r`
        else if command.starts_with("restart") {
            // TODO: Make this work
            window.write_to_command_line("Restart")?
        }
        // Remove saved sessions from the main screen
        else if command.starts_with('r') {
            if let StreamType::Auxiliary = window.config.stream_type {
                if let Ok(items) = self.resolve_delete_command(command) {
                    if let Some(del) = window.config.delete_func {
                        match del(&items) {
                            Ok(_) => {}
                            Err(why) => window.write_to_command_line(&why.to_string())?,
                        }
                        window.render_auxiliary_text()?;
                    } else {
                        {
                            window.write_to_command_line(
                                "Delete command is valid, but there is nothing to delete.",
                            )?;
                        }
                    }
                } else {
                    {
                        window.write_to_command_line(&format!(
                            "Failed to parse remove command: {:?} is invalid.",
                            command
                        ))?;
                    }
                }
            } else {
                {
                    window.write_to_command_line("Cannot remove files outside of startup mode.")?;
                }
            }
        }
        // Credits! Only accessible from the startup window
        else if command.starts_with("credits") {
            // Since getting here implies that we are now in command mode, check if the previous input type was startup
            if let InputType::Startup = window.previous_input_type {
                window.config.generate_auxiliary_messages = Some(gen);
                window.config.stream_type = StreamType::Auxiliary;
                window.config.scroll_state = ScrollState::Top;
                window.render_auxiliary_text()?;
                window.write_to_command_line("You've reached the credits! C-c or :q to exit.")?;
            }
        } else if command.starts_with("agg") {
            match self.resolve_aggregation_count(command) {
                Ok(val) => {
                    window.config.num_to_aggregate = val;
                    // TODO: This wont cause the screen to re-render until there is a new message to get parsed
                }
                Err(why) => {
                    window.write_to_command_line(&format!(
                        "Failed to parse aggregation count command: {:?}",
                        why
                    ))?;
                }
            }
        } else {
            window.write_to_command_line(&format!("Invalid command: {:?}", command))?;
        }
        self.return_to_prev_state(window)?;
        Ok(())
    }
}

impl Handler for CommandHandler {
    fn new() -> CommandHandler {
        CommandHandler {
            input_handler: UserInputHandler::new(),
        }
    }

    fn receive_input(&mut self, window: &mut MainWindow, key: KeyCode) -> Result<()> {
        match key {
            // Execute the command
            KeyCode::Enter => {
                let command = match self.input_handler.gather(window) {
                    Ok(command) => command,
                    Err(why) => panic!("Unable to gather text: {:?}", why),
                };
                self.process_command(window, &command)?;
            }
            // Go back to the previous state
            KeyCode::Esc => self.return_to_prev_state(window)?,
            key => self.input_handler.receive_input(window, key)?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod poll_rate_tests {
    use super::CommandHandler;
    use crate::communication::handlers::handler::Handler;

    #[test]
    fn test_can_set_poll_rate() {
        let handler = CommandHandler::new();
        let result = handler.resolve_poll_rate("poll 1");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_do_not_set_bad_poll_rate() {
        let handler = CommandHandler::new();
        let result = handler.resolve_poll_rate("poll v");
        assert!(result.is_err());
    }

    #[test]
    fn test_do_no_poll_rate() {
        let handler = CommandHandler::new();
        let result = handler.resolve_poll_rate("poll");
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod remove_tests {
    use super::CommandHandler;
    use crate::communication::handlers::handler::Handler;

    #[test]
    fn test_resolve_single_num() {
        let handler = CommandHandler::new();
        let resolved = handler.resolve_delete_command("r 1").unwrap_or_default();
        assert_eq!(resolved, [1]);
    }

    #[test]
    fn test_resolve_double_num() {
        let handler = CommandHandler::new();
        let resolved = handler.resolve_delete_command("r 1,2").unwrap_or_default();
        assert_eq!(resolved, [1, 2]);
    }

    #[test]
    fn test_resolve_triple_num() {
        let handler = CommandHandler::new();
        let resolved = handler
            .resolve_delete_command("r 1,2,3")
            .unwrap_or_default();
        assert_eq!(resolved, [1, 2, 3]);
    }

    #[test]
    fn test_resolve_triple_num_trailing_comma() {
        let handler = CommandHandler::new();
        let resolved = handler
            .resolve_delete_command("r 1,2,3,")
            .unwrap_or_default();
        assert_eq!(resolved, [1, 2, 3]);
    }

    #[test]
    fn test_resolve_range() {
        let handler = CommandHandler::new();
        let resolved = handler.resolve_delete_command("r 1-5").unwrap_or_default();
        assert_eq!(resolved, [1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_resolve_double_range() {
        let handler = CommandHandler::new();
        let resolved = handler
            .resolve_delete_command("r 1-3,5-7")
            .unwrap_or_default();
        assert_eq!(resolved, [1, 2, 3, 5, 6, 7]);
    }

    #[test]
    fn test_resolve_triple_range() {
        let handler = CommandHandler::new();
        let resolved = handler
            .resolve_delete_command("r 1-3,5-7,9-11")
            .unwrap_or_default();
        assert_eq!(resolved, [1, 2, 3, 5, 6, 7, 9, 10, 11]);
    }

    #[test]
    fn test_resolve_ranges_with_singletons() {
        let handler = CommandHandler::new();
        let resolved = handler
            .resolve_delete_command("r 1-3,5,9-11,15")
            .unwrap_or_default();
        assert_eq!(resolved, [1, 2, 3, 5, 9, 10, 11, 15]);
    }

    #[test]
    fn test_resolve_ranges_multiple_dash() {
        let handler = CommandHandler::new();
        let resolved = handler
            .resolve_delete_command("r 1--3,4")
            .unwrap_or_default();
        assert_eq!(resolved, [4]);
    }

    #[test]
    fn test_resolve_ranges_with_string() {
        let handler = CommandHandler::new();
        let resolved = handler
            .resolve_delete_command("r a-b,4")
            .unwrap_or_default();
        assert_eq!(resolved.len(), 0);
    }

    #[test]
    fn test_resolve_no_num() {
        let handler = CommandHandler::new();
        let resolved = handler.resolve_delete_command("r").unwrap_or_default();
        assert_eq!(resolved.len(), 0);
    }

    #[test]
    fn test_resolve_no_num_space() {
        let handler = CommandHandler::new();
        let resolved = handler.resolve_delete_command("r ").unwrap_or_default();
        assert_eq!(resolved.len(), 0);
    }
}
