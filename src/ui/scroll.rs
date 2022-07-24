use std::cmp::{max, min};

use crate::communication::reader::MainWindow;

#[derive(Debug)]
pub enum ScrollState {
    Top,
    Free,
    Bottom,
}

pub fn up(window: &mut MainWindow) {
    window.config.scroll_state = ScrollState::Free;

    // TODO: handle underflow
    window.config.current_end = match window.config.current_end.checked_sub(1) {
        Some(value) => max(value, 1), // No scrolling past the first message
        None => 1,
    };
}

pub fn down(window: &mut MainWindow) {
    window.config.scroll_state = ScrollState::Free;

    // Get number of messages we can scroll
    let num_messages = window.number_of_messages();

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
    window.config.scroll_state = ScrollState::Bottom
}

pub fn top(window: &mut MainWindow) {
    window.config.scroll_state = ScrollState::Top
}

#[cfg(test)]
mod tests {
    use crate::{
        communication::{input::input_type::InputType::Regex, reader::MainWindow},
        ui::scroll,
    };

    #[test]
    fn test_render_final_items_scroll_down() {
        let mut logria = MainWindow::_new_dummy();

        // Set scroll state
        logria.config.scroll_state = scroll::ScrollState::Bottom;

        // Set existing status
        logria.determine_render_position();

        // Scroll action
        scroll::down(&mut logria);

        let (start, end) = logria.determine_render_position();
        assert_eq!(start, 93);
        assert_eq!(end, 100);
    }

    #[test]
    fn test_render_first_items_scroll_up() {
        let mut logria = MainWindow::_new_dummy();

        // Set scroll state
        logria.config.scroll_state = scroll::ScrollState::Top;

        // Set existing status
        logria.determine_render_position();

        // Scroll action
        scroll::up(&mut logria);

        let (start, end) = logria.determine_render_position();
        assert_eq!(start, 0);
        assert_eq!(end, 6);
    }

    #[test]
    fn test_render_final_items_scroll_bottom() {
        let mut logria = MainWindow::_new_dummy();

        // Set scroll state
        logria.config.scroll_state = scroll::ScrollState::Bottom;

        // Set existing status
        logria.determine_render_position();

        // Scroll action
        scroll::bottom(&mut logria);

        let (start, end) = logria.determine_render_position();
        assert_eq!(start, 93);
        assert_eq!(end, 100);
    }

    #[test]
    fn test_render_first_items_scroll_top() {
        let mut logria = MainWindow::_new_dummy();

        // Set scroll state
        logria.config.scroll_state = scroll::ScrollState::Top;

        // Set existing status
        logria.determine_render_position();

        // Scroll action
        scroll::top(&mut logria);

        let (start, end) = logria.determine_render_position();
        assert_eq!(start, 0);
        assert_eq!(end, 7);
    }

    #[test]
    fn test_render_final_items_scroll_pgup() {
        let mut logria = MainWindow::_new_dummy();

        // Set scroll state
        logria.config.scroll_state = scroll::ScrollState::Bottom;

        // Set existing status
        logria.determine_render_position();

        // Scroll action
        scroll::pg_up(&mut logria);

        let (start, end) = logria.determine_render_position();
        assert_eq!(start, 86);
        assert_eq!(end, 93);
    }

    #[test]
    fn test_render_first_items_scroll_pgdn() {
        let mut logria = MainWindow::_new_dummy();

        // Set scroll state
        logria.config.scroll_state = scroll::ScrollState::Top;

        // Set existing status
        logria.determine_render_position();

        // Scroll action
        scroll::pg_down(&mut logria);

        let (start, end) = logria.determine_render_position();
        assert_eq!(start, 7);
        assert_eq!(end, 14);
    }

    #[test]
    fn test_render_scroll_past_end() {
        let mut logria = MainWindow::_new_dummy();

        // Set scroll state
        logria.config.scroll_state = scroll::ScrollState::Bottom;

        // Set existing status
        logria.config.current_end = 101; // somehow longer than the messages buffer

        // Scroll action
        scroll::down(&mut logria);

        let (start, end) = logria.determine_render_position();
        assert_eq!(start, 93);
        assert_eq!(end, 100);
    }

    #[test]
    fn test_render_scroll_past_end_small() {
        let mut logria = MainWindow::_new_dummy();

        // Set scroll state
        logria.config.scroll_state = scroll::ScrollState::Bottom;

        // Set existing status
        logria.config.current_end = 10;

        // Set state to regex mode
        logria.config.matched_rows = (0..5).collect();
        logria.config.regex_pattern = Some(regex::bytes::Regex::new("fa.ke").unwrap());
        logria.input_type = Regex;

        // Scroll action
        scroll::down(&mut logria);

        let (start, end) = logria.determine_render_position();
        assert_eq!(start, 0);
        assert_eq!(end, 5);
    }

    #[test]
    fn test_render_final_items_scroll_down_matched() {
        let mut logria = MainWindow::_new_dummy();

        // Set scroll state
        logria.config.scroll_state = scroll::ScrollState::Bottom;

        // Set existing status
        logria.determine_render_position();

        // Set state to regex mode
        logria.input_type = Regex;
        logria.config.regex_pattern = Some(regex::bytes::Regex::new("fa.ke").unwrap());
        logria.config.matched_rows = (0..20).collect();

        // Scroll action
        scroll::down(&mut logria);

        let (start, end) = logria.determine_render_position();
        assert_eq!(start, 13);
        assert_eq!(end, 20);
    }

    #[test]
    fn test_render_first_items_scroll_up_matched() {
        let mut logria = MainWindow::_new_dummy();

        // Set scroll state
        logria.config.scroll_state = scroll::ScrollState::Top;

        // Set existing status
        logria.determine_render_position();

        // Set state to regex mode
        logria.input_type = Regex;
        logria.config.matched_rows = (0..20).collect();

        // Scroll action
        scroll::up(&mut logria);

        let (start, end) = logria.determine_render_position();
        assert_eq!(start, 0);
        assert_eq!(end, 6);
    }
}
