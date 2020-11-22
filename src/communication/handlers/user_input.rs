use ncurses::{addch, curs_set, getmaxyx, mv, mvwinch, CURSOR_VISIBILITY};

use super::handler::HanderMethods;
use crate::communication::reader::main::MainWindow;

// Used in Command and Regex handler to capture user typing
pub struct UserInputHandler {
    x: i32,
    y: i32,
    last_write: i32,
}

impl UserInputHandler {
    /// Get the useable area of the textbox container
    fn update_dimensions(&mut self, window: &MainWindow) {
        getmaxyx(window.screen(), &mut self.y, &mut self.x)
    }

    /// Get the first and last spots we can write to on the command line
    fn get_first_last(&self) -> (i32, i32) {
        (1, self.x())
    }

    fn y(&self) -> i32 {
        self.y - 2
    }

    fn x(&self) -> i32 {
        self.x - 3
    }

    fn validate(&self, key: &i32) -> u32 {
        match key {
            127 => 263, // Ctrl-h to backspace
            key => *key as u32,
        }
    }

    /// Get the first empty x position in the command line
    fn end_of_line(&mut self, window: &MainWindow) -> i32 {
        // Ensure we are using the current screen size
        self.update_dimensions(window);

        // Get the last position we can write to
        let (_, mut last) = self.get_first_last();

        // Scan right to left
        loop {
            // Check for space, aka blank char
            if mvwinch(window.screen(), self.y(), last) != 32 {
                return std::cmp::min(last + 1, self.x());
            }

            // If we are at the first char, we have an empty cli
            if last == 1 {
                break;
            }

            last -= 1;
        }
        last
    }

    /// Insert character to the input window
    /// TODO: Support insert vs normal typing mode
    fn insert_char(&mut self, window: &MainWindow, character: u32) {
        // Ensure we are using the current screen size
        self.update_dimensions(window);

        // Last filled position
        if self.last_write == 0 {
            self.last_write = self.end_of_line(window);
        }

        // Handle movement
        if self.last_write < self.x() {
            // Move to the last written position
            mv(self.y(), self.last_write);

            // Insert the char
            addch(character);

            // Increment the last written position
            self.last_write += 1;
        }
    }

    /// Remove char 1 to the left of the cursor
    fn backspace(&self, window: &MainWindow) {}

    /// Remove char 1 to the right of the cursor
    fn delete(&self, window: &MainWindow) {}

    /// Get the contents of the command line as a string
    pub fn gather(&mut self, window: &MainWindow) -> String {
        let last_char = self.end_of_line(window);
        let mut out_s = String::from("");
        for position in 1..last_char {
            let char_at_pos = mvwinch(window.screen(), self.y(), position);
            out_s.push(self.get_char(char_at_pos));
        }
        curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

        // Reset the last written spot
        self.last_write = 0;
        window.reset_command_line();
        out_s
    }

    fn do_command(&mut self, window: &MainWindow, command: u32) -> bool {
        self.end_of_line(window);
        match command {
            127 => self.backspace(window),
            263 => self.backspace(window),
            command => self.insert_char(&window, command),
        }
        true
    }
}

impl HanderMethods for UserInputHandler {
    fn new() -> UserInputHandler {
        UserInputHandler {
            x: 0,
            y: 0,
            last_write: 0,
        }
    }

    fn recieve_input(&mut self, window: &mut MainWindow, key: i32) {
        curs_set(CURSOR_VISIBILITY::CURSOR_VISIBLE);
        let valid_key = self.validate(&key);
        let success = self.do_command(window, valid_key);
        // window.write_to_command_line("got data in CommandHandler");
    }
}
