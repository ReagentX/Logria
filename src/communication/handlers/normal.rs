use std::cmp::{max, min};

use super::handler::HanderMethods;
use crate::communication::input::input_type::InputType;
use crate::communication::input::stream_type::StreamType;
use crate::communication::reader::main::MainWindow;

pub struct NormalHandler {}

impl NormalHandler {
    fn scroll_up(&self, window: &mut MainWindow) {
        // TODO: Smart Poll Rate
        window.config.stick_to_top = false;
        window.config.stick_to_bottom = false;
        window.config.manually_controlled_line = true;

        // TODO: handle underflow
        window.config.current_end = match window.config.current_end.checked_sub(1) {
            Some(value) => max(value, 1), // No scrolling past the first message
            None => 1,
        };
    }

    fn scroll_down(&self, window: &mut MainWindow) {
        // TODO: Smart Poll Rate
        window.config.stick_to_top = false;
        window.config.stick_to_bottom = false;
        window.config.manually_controlled_line = true;

        // Get number of messages we can scroll
        let num_messages = match window.input_type {
            InputType::Normal | InputType::MultipleChoice | InputType::Command => {
                window.messages().len()
            }
            InputType::Parser | InputType::Regex => window.config.matched_rows.len(),
        };

        // No scrolling past the last message
        window.config.current_end = min(num_messages, window.config.current_end + 1);
    }

    fn pg_up(&self, window: &mut MainWindow) {
        (0..window.config.last_row).for_each(|_| self.scroll_up(window));
    }

    fn pg_down(&self, window: &mut MainWindow) {
        (0..window.config.last_row).for_each(|_| self.scroll_down(window));
    }

    fn bottom(&self, window: &mut MainWindow) {
        window.config.stick_to_top = false;
        window.config.stick_to_bottom = true;
        window.config.manually_controlled_line = false;
    }

    fn top(&self, window: &mut MainWindow) {
        window.config.stick_to_top = true;
        window.config.stick_to_bottom = false;
        window.config.manually_controlled_line = false;
    }

    fn set_command_mode(&self, window: &mut MainWindow) {
        window.input_type = InputType::Command;
    }

    fn set_parser_mode(&self, window: &mut MainWindow) {
        window.input_type = InputType::Parser;
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

    fn noop(&self) {}
}

impl HanderMethods for NormalHandler {
    fn new() -> NormalHandler {
        NormalHandler {}
    }

    fn recieve_input(&mut self, window: &mut MainWindow, key: i32) {
        match key {
            258 => self.scroll_down(window),     // down
            259 => self.scroll_up(window),       // up
            260 => self.top(window),             // left
            261 => self.bottom(window),          // right
            262 => self.top(window),             // home
            263 => self.bottom(window),          // end
            338 => self.pg_down(window),         // pgdn
            339 => self.pg_up(window),           // pgup
            58 => self.set_command_mode(window), // /
            47 => self.set_regex_mode(window),   // :
            112 => self.set_parser_mode(window), // p
            115 => self.swap_streams(window),    // s
            _ => self.noop(),
        }
    }
}
