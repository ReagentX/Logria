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
    content: String,
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

    /// Insert character to the input window
    /// TODO: Support insert vs normal typing mode
    fn insert_char(&mut self, window: &mut MainWindow, character: KeyCode) {
        match character {
            KeyCode::Char(c) => {
                // Ensure we are using the current screen size
                self.update_dimensions();

                // Handle movement
                if self.last_write < self.x() {
                    // Insert the char
                    queue!(
                        window.output,
                        cursor::MoveTo(self.last_write, self.y()),
                        style::Print(c)
                    )
                    .unwrap();

                    // Increment the last written position
                    self.last_write += 1;

                    self.content.push(c)
                }
            }
            _ => {}
        }
    }

    /// Remove char 1 to the left of the cursor
    fn backspace(&self, window: &MainWindow) {}

    /// Remove char 1 to the right of the cursor
    fn delete(&self, window: &MainWindow) {}

    /// Get the contents of the command line as a string
    pub fn gather(&mut self, window: &mut MainWindow) -> String {
        // Copy the result to a new place so we can clear out the existing one and reuse the struct
        let result = String::from(&self.content);
        self.content = String::new();

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
            command => self.insert_char(window, command),
        }
        window.output.flush().unwrap();
        true
    }
}

impl HanderMethods for UserInputHandler {
    fn new() -> UserInputHandler {
        UserInputHandler {
            x: 0,
            y: 0,
            last_write: 1,
            content: String::new(),
        }
    }

    fn recieve_input(&mut self, window: &mut MainWindow, key: KeyCode) {
        queue!(window.output, cursor::Hide).unwrap();
        let success = self.do_command(window, key);
    }
}
