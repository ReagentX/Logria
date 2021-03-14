use std::{collections::HashMap, io::Write};

use crossterm::{cursor, event::KeyCode, queue, Result};

use super::{handler::HanderMethods, user_input::UserInputHandler};
use crate::{
    communication::{
        input::{input_type::InputType, stream::build_streams, stream_type::StreamType::StdErr},
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
    fn get_startup_text(&mut self) -> Vec<String> {
        let mut text: Vec<String> = Vec::new();
        let sessions = Session::list();
        START_MESSAGE.iter().for_each(|&s| text.push(s.to_string()));
        sessions.iter().enumerate().for_each(|(i, s)| {
            let value = s.to_string();
            text.push(format!("{}: {}", i, value));
            self.session_data.insert(i, value);
        });
        text
    }

    /// Load the window's startup text buffer
    pub fn render_startup_text(&mut self, window: &mut MainWindow) -> Result<()> {
        window.config.startup_messages = self.get_startup_text();
        Ok(())
    }

    fn set_command_mode(&self, window: &mut MainWindow) -> Result<()> {
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
                                window.config.streams = build_streams(session.commands);
                                window.config.stream_type = StdErr;
                                window.input_type = InputType::Normal;
                            },
                            Err(why) => {
                                window.write_to_command_line(&format!("Unable to parse session: {:?}", why))?;
                            }
                        }
                    }
                    None => {
                        window.write_to_command_line("Invalid selection!")?;
                    }
                }
                return Ok(());
            }
            Err(_) => {
                window.write_to_command_line("Invalid selection!")?;
            }
        }
        Ok(())
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
