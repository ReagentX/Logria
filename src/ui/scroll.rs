use std::cmp::min;

use crate::{communication::reader::MainWindow, util::binary_search::closest_index};

#[derive(Debug)]
pub enum ScrollState {
    Top,
    Free,
    Bottom,
    Centered,
}

pub fn up(window: &mut MainWindow) {
    window.config.scroll_state = ScrollState::Free;

    window.config.current_end = window.config.current_end.saturating_sub(1).max(1);
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
    window.config.scroll_state = ScrollState::Bottom;
}

pub fn top(window: &mut MainWindow) {
    window.config.scroll_state = ScrollState::Top;
}

/// Scroll to the current matched row in the matched rows vector, used by the highlight search
pub fn update_current_match_index(window: &mut MainWindow, scroll_up: bool) {
    match window.config.scroll_state {
        ScrollState::Free => {
            let (start, end) = window.config.previous_render;
            // The midpoint index of the current render
            let render_midpoint = (start + end) / 2;
            // Find the closest match to the render midpoint in the matched rows
            window.config.current_matched_row =
                closest_index(&window.config.matched_rows, render_midpoint)
                    .unwrap_or(window.config.current_matched_row);
            // Only change the scroll state if there is a match to render
            if !window.config.matched_rows.is_empty() {
                window.config.scroll_state = ScrollState::Centered;
            }
        }
        ScrollState::Top => {
            // If we are at the top, we can just return to the first match
            window.config.current_matched_row = 0;
            // Only change the scroll state if there is a match to render
            if !window.config.matched_rows.is_empty() {
                window.config.scroll_state = ScrollState::Centered;
            }
        }
        ScrollState::Bottom => {
            // If we are at the bottom, we can just return to the last match
            window.config.current_matched_row = window.config.matched_rows.len().saturating_sub(1);
            // Only change the scroll state if there is a match to render
            if !window.config.matched_rows.is_empty() {
                window.config.scroll_state = ScrollState::Centered;
            }
        }
        ScrollState::Centered => {
            if scroll_up {
                // Scrolling back one match, up to the first index in the list
                window.config.current_matched_row =
                    window.config.current_matched_row.saturating_sub(1);
            } else {
                // Scrolling forward one match, up to the last index in the list
                window.config.current_matched_row = min(
                    window.config.current_matched_row + 1,
                    window.config.matched_rows.len().saturating_sub(1),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        communication::{input::InputType::Regex, reader::MainWindow},
        ui::scroll::{self, ScrollState},
    };

    #[test]
    fn test_render_final_items_scroll_down() {
        let mut logria = MainWindow::_new_dummy();

        // Set scroll state
        logria.config.scroll_state = ScrollState::Bottom;

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
        logria.config.scroll_state = ScrollState::Top;

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
        logria.config.scroll_state = ScrollState::Bottom;

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
        logria.config.scroll_state = ScrollState::Top;

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
        logria.config.scroll_state = ScrollState::Bottom;

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
        logria.config.scroll_state = ScrollState::Top;

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
        logria.config.scroll_state = ScrollState::Bottom;

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
        logria.config.scroll_state = ScrollState::Bottom;

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
        logria.config.scroll_state = ScrollState::Bottom;

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
        logria.config.scroll_state = ScrollState::Top;

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

    #[test]
    fn test_update_current_match_index_free_closest_high() {
        let mut window = MainWindow::_new_dummy();
        window.config.matched_rows = vec![5, 15, 25];
        window.config.current_matched_row = 99;
        window.config.previous_render = (0, 22); // midpoint is 11
        window.config.scroll_state = ScrollState::Free;

        scroll::update_current_match_index(&mut window, false);

        assert_eq!(window.config.current_matched_row, 1);
        assert!(matches!(window.config.scroll_state, ScrollState::Centered));
    }

    #[test]
    fn test_update_current_match_index_free_closest_low() {
        let mut window = MainWindow::_new_dummy();
        window.config.matched_rows = vec![5, 15, 25];
        window.config.current_matched_row = 99;
        window.config.previous_render = (0, 21); // midpoint is 10
        window.config.scroll_state = ScrollState::Free;

        scroll::update_current_match_index(&mut window, false);

        assert_eq!(window.config.current_matched_row, 0);
        assert!(matches!(window.config.scroll_state, ScrollState::Centered));
    }

    #[test]
    fn test_update_current_match_index_free_empty() {
        let mut window = MainWindow::_new_dummy();
        window.config.matched_rows.clear();
        window.config.current_matched_row = 3;
        window.config.previous_render = (0, 50);
        window.config.scroll_state = ScrollState::Free;

        scroll::update_current_match_index(&mut window, true);

        assert_eq!(window.config.current_matched_row, 3);
        assert!(matches!(window.config.scroll_state, ScrollState::Centered));
    }

    #[test]
    fn test_update_current_match_index_top() {
        let mut window = MainWindow::_new_dummy();
        window.config.matched_rows = vec![1, 2, 3];
        window.config.current_matched_row = 5;
        window.config.scroll_state = ScrollState::Top;

        scroll::update_current_match_index(&mut window, false);

        assert_eq!(window.config.current_matched_row, 0);
        assert!(matches!(window.config.scroll_state, ScrollState::Centered));
    }

    #[test]
    fn test_update_current_match_index_bottom() {
        let mut window = MainWindow::_new_dummy();
        window.config.matched_rows = vec![1, 2, 3, 4];
        window.config.current_matched_row = 0;
        window.config.scroll_state = ScrollState::Bottom;

        scroll::update_current_match_index(&mut window, false);

        assert_eq!(window.config.current_matched_row, 3);
        assert!(matches!(window.config.scroll_state, ScrollState::Centered));
    }

    #[test]
    fn test_update_current_match_index_centered_down() {
        let mut window = MainWindow::_new_dummy();
        window.config.matched_rows = vec![0, 1, 2];
        window.config.current_matched_row = 1;
        window.config.scroll_state = ScrollState::Centered;

        scroll::update_current_match_index(&mut window, false);

        assert_eq!(window.config.current_matched_row, 2);
    }

    #[test]
    fn test_update_current_match_index_centered_up() {
        let mut window = MainWindow::_new_dummy();
        window.config.matched_rows = vec![0, 1, 2];
        window.config.current_matched_row = 1;
        window.config.scroll_state = ScrollState::Centered;

        scroll::update_current_match_index(&mut window, true);

        assert_eq!(window.config.current_matched_row, 0);
    }
}
