use std::cmp::{max, min};

use crate::communication::reader::main::MainWindow;

pub fn up(window: &mut MainWindow) {
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

pub fn down(window: &mut MainWindow) {
    // TODO: Smart Poll Rate
    window.config.stick_to_top = false;
    window.config.stick_to_bottom = false;
    window.config.manually_controlled_line = true;

    // Get number of messages we can scroll
    let num_messages = window.numer_of_messages();

    // No scrolling past the last message
    window.config.current_end = min(num_messages, window.config.current_end + 1);
}

pub fn pg_up(window: &mut MainWindow) {
    (0..window.config.last_row).for_each(|_| up(window));
}

pub fn pg_down(window: &mut MainWindow) {
    (0..window.config.last_row).for_each(|_| down(window));
}

pub fn bottom(window: &mut MainWindow) {
    window.config.stick_to_top = false;
    window.config.stick_to_bottom = true;
    window.config.manually_controlled_line = false;
}

pub fn top(window: &mut MainWindow) {
    window.config.stick_to_top = true;
    window.config.stick_to_bottom = false;
    window.config.manually_controlled_line = false;
}
