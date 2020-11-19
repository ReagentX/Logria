pub mod build {
    use ncurses::{initscr, endwin, noecho, cbreak, nocbreak, start_color, use_default_colors, echo, keypad, newwin, mvwvline, mvwhline, mvwaddch};

    use crate::communication::reader::main::LogiraConfig;

    pub fn init_scr() -> ncurses::WINDOW {
        // Initialize curses
        let stdscr = initscr();

        // Turn off echoing of keys, and enter cbreak mode,
        // where no buffering is performed on keyboard input
        noecho();
        cbreak();

        // Start color, too.  Harmless if the terminal doesn't have
        // color; user can test with has_color() later on.  The try/catch
        // works around a minor bit of over-conscientiousness in the curses
        // module -- the error return from C start_color() is ignorable.
        start_color();
        use_default_colors();

        // In keypad mode, escape sequences for special keys
        // (like the cursor keys) will be interpreted and
        // a special value like curses.KEY_LEFT will be returned
        keypad(stdscr, true);

        // Give the caller back the pointer
        stdscr
    }

    pub fn exit_scr(stdscr: ncurses::WINDOW) {
        // Set everything back to normal
        keypad(stdscr, false);
        echo();
        nocbreak();
        endwin();
    }

    fn rectangle(stdscr: ncurses::WINDOW, uly: i32, ulx: i32, lry: i32, lrx: i32) {
        mvwvline(stdscr, uly+1, ulx, ncurses::ACS_VLINE(), lry - uly - 1);
        mvwhline(stdscr, uly, ulx+1, ncurses::ACS_HLINE(), lrx - ulx - 1);
        mvwhline(stdscr, lry, ulx+1, ncurses::ACS_HLINE(), lrx - ulx - 1);
        mvwvline(stdscr, uly+1, lrx, ncurses::ACS_VLINE(), lry - uly - 1);
        mvwaddch(stdscr, uly, ulx, ncurses::ACS_ULCORNER());
        mvwaddch(stdscr, uly, lrx, ncurses::ACS_URCORNER());
        mvwaddch(stdscr, lry, lrx, ncurses::ACS_LRCORNER());
        mvwaddch(stdscr, lry, ulx, ncurses::ACS_LLCORNER());
    }

    pub fn command_line(stdscr: ncurses::WINDOW, config: &LogiraConfig) -> ncurses::WINDOW {
        // 1 line, screen width, start 2 from the bottom, 1 char from the side
        let window = newwin(1, config.width, config.height - 2, 1);
        
        // Do not block the event loop waiting for input
        ncurses::nodelay(window, true);

        // Draw rectangle around the command line
        // upper left:  (height - 2, 0), 2 chars up on left edge
        // lower right: (height, width), bottom right corner of screen
        rectangle(stdscr, config.height - 3, 0, config.height - 1, config.width - 2);
        ncurses::refresh();
        window
    }

}
