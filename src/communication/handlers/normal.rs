use crossterm::event::KeyCode;

use super::handler::HanderMethods;
use crate::communication::input::input_type::InputType;
use crate::communication::input::stream_type::StreamType;
use crate::communication::reader::main::MainWindow;

pub struct NormalHandler {}

impl NormalHandler {
    fn set_command_mode(&self, window: &mut MainWindow) {
        window.input_type = InputType::Command;
        window.set_cli_cursor(None);
    }

    fn set_parser_mode(&self, window: &mut MainWindow) {
        window.input_type = InputType::Parser;
        window.set_cli_cursor(None);
    }

    fn set_regex_mode(&self, window: &mut MainWindow) {
        window.input_type = InputType::Regex;
        window.set_cli_cursor(None);
    }

    fn swap_streams(&self, window: &mut MainWindow) {
        window.config.stream_type = match window.config.stream_type {
            StreamType::StdOut => StreamType::StdErr,
            StreamType::StdErr => StreamType::StdOut,
        }
    }
}

impl HanderMethods for NormalHandler {
    fn new() -> NormalHandler {
        NormalHandler {}
    }

    fn recieve_input(&mut self, window: &mut MainWindow, key: KeyCode) {
        match key {
            KeyCode::Char(':') => self.set_command_mode(window),
            KeyCode::Char('/') => self.set_regex_mode(window),
            KeyCode::Char('p') => self.set_parser_mode(window),
            KeyCode::Char('s') => self.swap_streams(window),
            _ => {}
        }
    }
}
