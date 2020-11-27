use crossterm::event::KeyCode;
use crossterm::Result;

use super::handler::HanderMethods;
use crate::communication::input::input_type::InputType;
use crate::communication::input::stream_type::StreamType;
use crate::communication::reader::main::MainWindow;
use crate::ui::scroll;

pub struct NormalHandler {}

impl NormalHandler {
    fn set_command_mode(&self, window: &mut MainWindow) -> Result<()> {
        window.input_type = InputType::Command;
        window.reset_command_line()?;
        window.set_cli_cursor(None)?;
        Ok(())
    }

    fn set_parser_mode(&self, window: &mut MainWindow) -> Result<()> {
        window.input_type = InputType::Parser;
        window.set_cli_cursor(None)?;
        Ok(())
    }

    fn set_regex_mode(&self, window: &mut MainWindow) -> Result<()> {
        window.input_type = InputType::Regex;
        window.config.highlight_match = true;
        window.reset_command_line()?;
        window.set_cli_cursor(None)?;
        Ok(())
    }

    fn swap_streams(&self, window: &mut MainWindow) -> Result<()> {
        window.config.stream_type = match window.config.stream_type {
            StreamType::StdOut => StreamType::StdErr,
            StreamType::StdErr => StreamType::StdOut,
        };
        window.input_type = InputType::Normal;
        window.set_cli_cursor(None)?;
        Ok(())
    }
}

impl HanderMethods for NormalHandler {
    fn new() -> NormalHandler {
        NormalHandler {}
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

            // Modes
            KeyCode::Char(':') => self.set_command_mode(window)?,
            KeyCode::Char('/') => self.set_regex_mode(window)?,
            KeyCode::Char('p') => self.set_parser_mode(window)?,
            KeyCode::Char('s') => self.swap_streams(window)?,
            _ => {}
        }
        Ok(())
    }
}
