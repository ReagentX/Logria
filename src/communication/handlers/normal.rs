use crossterm::{cursor, event::KeyCode, queue, Result};

use super::handler::Handler;
use crate::{
    communication::{
        input::{input_type::InputType, stream_type::StreamType},
        reader::main::MainWindow,
    },
    ui::scroll,
};

pub struct NormalHandler {}

impl NormalHandler {
    fn set_parser_mode(&self, window: &mut MainWindow) -> Result<()> {
        window.update_input_type(InputType::Parser)?;
        window.reset_command_line()?;
        window.set_cli_cursor(None)?;
        window.config.previous_stream_type = window.config.stream_type;
        window.config.stream_type = StreamType::Auxiliary;
        // Send 2 new refresh ticks from the main app loop when this method returns
        window.config.did_switch = true;
        Ok(())
    }

    fn set_regex_mode(&self, window: &mut MainWindow) -> Result<()> {
        window.go_to_cli()?;
        window.update_input_type(InputType::Regex)?;
        window.config.highlight_match = true;
        window.reset_command_line()?;
        window.set_cli_cursor(None)?;
        queue!(window.output, cursor::Show)?;
        // Send 2 new refresh ticks from the main app loop when this method returns
        window.config.did_switch = true;
        Ok(())
    }

    fn swap_streams(&self, window: &mut MainWindow) -> Result<()> {
        window.config.previous_stream_type = window.config.stream_type;
        window.config.stream_type = match window.config.stream_type {
            StreamType::StdOut => StreamType::StdErr,
            StreamType::StdErr => StreamType::StdOut,
            // Do not swap from auxiliary stream
            StreamType::Auxiliary => StreamType::Auxiliary,
        };
        window.update_input_type(InputType::Normal)?;
        window.set_cli_cursor(None)?;
        window.reset_command_line()?;
        window.reset_output()?;
        window.redraw()?;
        Ok(())
    }
}

impl Handler for NormalHandler {
    fn new() -> NormalHandler {
        NormalHandler {}
    }

    fn receive_input(&mut self, window: &mut MainWindow, key: KeyCode) -> Result<()> {
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
            KeyCode::Char(':') => window.set_command_mode(None)?,
            KeyCode::Char('/') => self.set_regex_mode(window)?,
            KeyCode::Char('p') => self.set_parser_mode(window)?,
            KeyCode::Char('s') => self.swap_streams(window)?,
            _ => {}
        }
        window.redraw()?;
        Ok(())
    }
}
