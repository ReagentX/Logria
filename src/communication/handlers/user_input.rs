use std::cmp::min;
use std::io::Write;

use crossterm::event::KeyCode;
use crossterm::terminal::size;
use crossterm::{cursor, queue, style};

use super::handler::HanderMethods;
use crate::communication::reader::main::MainWindow;

// Used in Command and Regex handler to capture user typing
pub struct UserInputHandler {
    x: u16,
    y: u16,
    last_write: u16,
    content: Vec<char>,
}

impl UserInputHandler {
    /// Get the useable area of the textbox container
    fn update_dimensions(&mut self) {
        let (w, h) = size().unwrap();
        self.y = h;
        self.x = w;
    }

    /// Get the first and last spots we can write to on the command line
    fn get_first_last(&self) -> (u16, u16) {
        (1, self.x())
    }

    fn y(&self) -> u16 {
        self.y - 2
    }

    fn x(&self) -> u16 {
        self.x - 3
    }

    fn get_content(&self) -> String {
        self.content.iter().collect()
    }

    fn write(&self, window: &mut MainWindow) {
        // Insert the word to the screen
        queue!(
            window.output,
            cursor::MoveTo(1, self.y()),
            style::Print(self.get_content()),
            cursor::MoveTo(self.last_write, self.y()),
        );
        window.output.flush();
    }

    /// Insert character to the input window
    /// TODO: Support insert vs normal typing mode
    fn insert_char(&mut self, window: &mut MainWindow, character: KeyCode) {
        match character {
            KeyCode::Char(c) => {
                // Ensure we are using the current screen size
                self.update_dimensions();

                // Handle movement
                if self.last_write < self.x() {
                    // Add the char to our data
                    self.content.insert(self.position_as_index(), c);

                    // Increment the last written position
                    self.last_write += 1;

                    // Insert the word to the screen
                    self.write(window);
                }
            }
            _ => {}
        }
    }

    fn position_as_index(&self) -> usize {
        (self.last_write - 1) as usize
    }

    /// Remove char 1 to the left of the cursor
    fn backspace(&mut self, window: &mut MainWindow) {
        if self.last_write >= 1 && self.content.len() > 0 {
            self.content.remove(self.position_as_index() - 1);
            self.move_left(window);
            self.write(window);
        }
    }

    /// Remove char 1 to the right of the cursor
    fn delete(&mut self, window: &mut MainWindow) {
        if self.last_write < self.x() && self.content.len() > 0 {
            self.content.remove(self.position_as_index());
            self.write(window);
        }
    }

    /// Move the cursor left
    fn move_left(&mut self, window: &mut MainWindow) {
        self.last_write = self.last_write.checked_sub(1).unwrap_or(0);
        queue!(
            window.output,
            cursor::MoveTo(self.last_write, self.y()),
        );
    }

    /// Move the cursor right
    fn move_right(&mut self, window: &mut MainWindow) {
        self.last_write = min(self.content.len() as u16, self.last_write + 1);
        queue!(
            window.output,
            cursor::MoveTo(self.last_write, self.y()),
        );
    }

    /// Get the contents of the command line as a string
    pub fn gather(&mut self, window: &mut MainWindow) -> String {
        // Copy the result to a new place so we can clear out the existing one and reuse the struct
        let result: String = self.get_content();
        self.content = vec![];

        // Hide the cursor
        queue!(window.output, cursor::Hide).unwrap();

        // Reset the last written spot
        self.last_write = 1;
        window.reset_command_line();

        result
    }

    fn do_command(&mut self, window: &mut MainWindow, command: KeyCode) -> bool {
        match command {
            KeyCode::Delete => self.delete(window),
            KeyCode::Backspace => self.backspace(window),
            KeyCode::Left => self.move_left(window),
            KeyCode::Right => self.move_right(window),
            // Possibly opt+left to skip words/symbols
            command => self.insert_char(window, command),
        }
        window.output.flush();
        true
    }
}

impl HanderMethods for UserInputHandler {
    fn new() -> UserInputHandler {
        UserInputHandler {
            x: 0,
            y: 0,
            last_write: 1,
            content: vec![],
        }
    }

    fn recieve_input(&mut self, window: &mut MainWindow, key: KeyCode) {
        queue!(window.output, cursor::Show).unwrap();
        let success = self.do_command(window, key);
    }
}
