pub mod build {
    use ncurses::{initscr, endwin, noecho, cbreak, nocbreak, start_color, use_default_colors, echo, keypad};

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

}

pub mod textbox {

}
