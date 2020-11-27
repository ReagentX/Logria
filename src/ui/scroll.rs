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

#[cfg(test)]
mod tests {
    use crate::communication::reader::main::MainWindow;
    use crate::communication::input::input_type::InputType::Regex;
    use crate::ui::scroll;

    #[test]
    fn test_render_final_items_scroll_down() {
        let mut logria = MainWindow::_new_dummy();

        // Set scroll state
        logria.config.manually_controlled_line = false;
        logria.config.stick_to_top = false;
        logria.config.stick_to_bottom = true;

        // Set existing status
        logria.determine_render_position();

        // Scroll action
        scroll::down(&mut logria);

        let (start, end) = logria.determine_render_position();
        assert_eq!(start, 92);
        assert_eq!(end, 100);
    }

    #[test]
    fn test_render_first_items_scroll_up() {
        let mut logria = MainWindow::_new_dummy();

        // Set scroll state
        logria.config.manually_controlled_line = false;
        logria.config.stick_to_top = true;
        logria.config.stick_to_bottom = false;

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
        logria.config.manually_controlled_line = false;
        logria.config.stick_to_top = false;
        logria.config.stick_to_bottom = true;

        // Set existing status
        logria.determine_render_position();

        // Scroll action
        scroll::bottom(&mut logria);

        let (start, end) = logria.determine_render_position();
        assert_eq!(start, 92);
        assert_eq!(end, 100);
    }

    #[test]
    fn test_render_first_items_scroll_top() {
        let mut logria = MainWindow::_new_dummy();

        // Set scroll state
        logria.config.manually_controlled_line = false;
        logria.config.stick_to_top = true;
        logria.config.stick_to_bottom = false;

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
        logria.config.manually_controlled_line = false;
        logria.config.stick_to_top = false;
        logria.config.stick_to_bottom = true;

        // Set existing status
        logria.determine_render_position();

        // Scroll action
        scroll::pg_up(&mut logria);

        let (start, end) = logria.determine_render_position();
        assert_eq!(start, 85);
        assert_eq!(end, 93);
    }

    #[test]
    fn test_render_first_items_scroll_pgdn() {
        let mut logria = MainWindow::_new_dummy();

        // Set scroll state
        logria.config.manually_controlled_line = false;
        logria.config.stick_to_top = true;
        logria.config.stick_to_bottom = false;

        // Set existing status
        logria.determine_render_position();

        // Scroll action
        scroll::pg_down(&mut logria);

        let (start, end) = logria.determine_render_position();
        assert_eq!(start, 6);
        assert_eq!(end, 14);
    }

    #[test]
    fn test_render_unset_scroll_state() {
        let mut logria = MainWindow::_new_dummy();

        // Set scroll state
        logria.config.manually_controlled_line = false;
        logria.config.stick_to_top = false;
        logria.config.stick_to_bottom = false;

        let (start, end) = logria.determine_render_position();
        assert_eq!(start, 92);
        assert_eq!(end, 100);
    }

    #[test]
    fn test_render_scroll_past_end() {
        let mut logria = MainWindow::_new_dummy();

        // Set scroll state
        logria.config.manually_controlled_line = false;
        logria.config.stick_to_top = false;
        logria.config.stick_to_bottom = true;

        // Set existing status
        logria.config.current_end = 101; // somehow longer than the messages buffer

        // Scroll action
        scroll::down(&mut logria);

        let (start, end) = logria.determine_render_position();
        assert_eq!(start, 92);
        assert_eq!(end, 100);
    }

    #[test]
    fn test_render_scroll_past_end_small() {
        let mut logria = MainWindow::_new_dummy();

        // Set scroll state
        logria.config.manually_controlled_line = false;
        logria.config.stick_to_top = false;
        logria.config.stick_to_bottom = true;

        // Set existing status
        logria.config.current_end = 10;
        
        // Set state to regex mode
        logria.config.matched_rows = (0..5).collect();
        logria.config.regex_pattern = Some(regex::bytes::Regex::new("").unwrap());
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
        logria.config.manually_controlled_line = false;
        logria.config.stick_to_top = false;
        logria.config.stick_to_bottom = true;

        // Set existing status
        logria.determine_render_position();

        // Set state to regex mode
        logria.input_type = Regex;
        logria.config.regex_pattern = Some(regex::bytes::Regex::new("").unwrap());
        logria.config.matched_rows = (0..20).collect();
        
        // Scroll action
        scroll::down(&mut logria);

        let (start, end) = logria.determine_render_position();
        assert_eq!(start, 12);
        assert_eq!(end, 20);
    }

    #[test]
    fn test_render_first_items_scroll_up_matched() {
        let mut logria = MainWindow::_new_dummy();

        // Set scroll state
        logria.config.manually_controlled_line = false;
        logria.config.stick_to_top = true;
        logria.config.stick_to_bottom = false;

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
