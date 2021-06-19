use std::{collections::HashMap, io::Write};

use crossterm::{cursor, event::KeyCode, queue, Result};

use super::{handler::HanderMethods, user_input::UserInputHandler};
use crate::{
    communication::{
        input::{
            input_type::InputType,
            stream::{build_streams_from_input, build_streams_from_session},
            stream_type::StreamType::StdErr,
        },
        reader::main::MainWindow,
    },
    constants::cli::messages::START_MESSAGE,
    extensions::session::Session,
    ui::scroll,
};

pub struct StartupHandler {
    input_handler: UserInputHandler,
    session_data: HashMap<usize, String>,
}

impl StartupHandler {
    /// Generate the startup message with available session configurations
    pub fn get_startup_text() -> Vec<String> {
        let mut text: Vec<String> = Vec::new();
        let sessions = Session::list();
        START_MESSAGE.iter().for_each(|&s| text.push(s.to_string()));
        sessions.iter().enumerate().for_each(|(i, s)| {
            let value = s.to_string();
            text.push(format!("{}: {}", i, value));
        });
        text
    }

    /// Load the session_data hashmap internally
    fn initialize(&mut self) {
        let sessions = Session::list();
        sessions.iter().enumerate().for_each(|(i, s)| {
            let value = s.to_string();
            self.session_data.insert(i, value);
        });
    }

    fn set_command_mode(&self, window: &mut MainWindow) -> Result<()> {
        // Allow the user to input commands so they quit and delete sessions
        window.go_to_cli()?;
        window.input_type = InputType::Command;
        window.reset_command_line()?;
        window.set_cli_cursor(None)?;
        queue!(window.output, cursor::Show)?;
        Ok(())
    }

    fn process_command(&mut self, window: &mut MainWindow, command: &str) -> Result<()> {
        let selection = command.parse::<usize>();
        match selection {
            Ok(item) => {
                match self.session_data.get(&item) {
                    Some(file_path) => {
                        let session = Session::load(file_path);
                        match session {
                            Ok(session) => {
                                window.config.streams = build_streams_from_session(session);
                                window.config.stream_type = StdErr;
                                window.input_type = InputType::Normal;
                                window.reset_output()?;
                                window.redraw()?;
                            }
                            Err(why) => {
                                window.write_to_command_line(&format!(
                                    "Unable to parse session: {:?}",
                                    why
                                ))?;
                            }
                        }
                    }
                    None => {
                        window.write_to_command_line("Invalid selection!")?;
                    }
                }
                Ok(())
            }
            Err(_) => {
                window.config.streams = build_streams_from_input(&vec![command.to_owned()], true);
                window.config.stream_type = StdErr;
                window.input_type = InputType::Normal;
                window.reset_output()?;
                window.redraw()?;
                Ok(())
            }
        }
    }
}

impl HanderMethods for StartupHandler {
    fn new() -> StartupHandler {
        StartupHandler {
            input_handler: UserInputHandler::new(),
            session_data: HashMap::new(),
        }
    }

    fn recieve_input(&mut self, window: &mut MainWindow, key: KeyCode) -> Result<()> {
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

            // Mode change for remove or config commands
            KeyCode::Char(':') => self.set_command_mode(window)?,

            // Handle user input selection
            KeyCode::Enter => {
                // Ensure the hashmap of files is updated
                self.initialize();
                let command = match self.input_handler.gather(window) {
                    Ok(command) => command,
                    Err(why) => panic!("Unable to gather text: {:?}", why),
                };
                self.process_command(window, &command)?;
            }

            // User input
            key => self.input_handler.recieve_input(window, key)?,
        }
        window.redraw()?;
        Ok(())
    }
}

#[cfg(test)]
mod startup_tests {
    use crate::{
        communication::{
            handlers::handler::HanderMethods,
            input::{input_type::InputType, stream_type::StreamType},
            reader::main::MainWindow,
        },
        constants::cli::messages::START_MESSAGE,
        extensions::session::Session,
    };

    use super::StartupHandler;

    #[test]
    fn can_initialize() {
        let mut handler = StartupHandler::new();
        handler.initialize();
    }

    #[test]
    fn can_get_startup_text() {
        let text = StartupHandler::get_startup_text();
        let sessions = Session::list();
        assert_eq!(text.len(), sessions.len() + START_MESSAGE.len())
    }

    #[test]
    fn can_load_session() {
        // Setup dummy window
        let mut window = MainWindow::_new_dummy();

        // Setup handler
        let mut handler = StartupHandler::new();
        handler.initialize();

        // Tests
        assert!(handler.process_command(&mut window, "0").is_ok());
        assert!(matches!(window.input_type, InputType::Normal));
        assert!(matches!(window.config.stream_type, StreamType::StdErr));
    }

    #[test]
    fn doesnt_crash_bad_index() {
        // Setup dummy window
        let mut window = MainWindow::_new_dummy();
        window.config.stream_type = StreamType::Auxiliary;

        // Setup handler
        let mut handler = StartupHandler::new();
        handler.initialize();

        // Tests
        assert!(handler.process_command(&mut window, "999").is_ok());
        assert!(matches!(window.input_type, InputType::Startup));
        assert!(matches!(window.config.stream_type, StreamType::Auxiliary));
    }

    #[test]
    fn doesnt_crash_alpha() {
        // Setup dummy window
        let mut window = MainWindow::_new_dummy();
        window.config.stream_type = StreamType::Auxiliary;

        // Setup handler
        let mut handler = StartupHandler::new();
        handler.initialize();

        // Tests
        assert!(handler.process_command(&mut window, "zzzfake_file_name").is_ok());
        assert!(matches!(window.input_type, InputType::Normal));
        assert!(matches!(window.config.stream_type, StreamType::StdErr));
        Session::del(&vec![Session::list().len() - 1]);
    }
}
