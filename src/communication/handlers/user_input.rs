use std::{
    cmp::{max, min},
    io::{stdout, Write},
};

use crossterm::{cursor, event::KeyCode, queue, style, terminal::size, Result};

use crate::{
    communication::{handlers::handler::Handler, reader::MainWindow},
    util::history::Tape,
};

// Used in Command and Regex handler to capture user typing
pub struct UserInputHandler {
    x: u16,
    y: u16,
    last_write: u16,
    content: Vec<char>,
    history: Tape,
}

impl UserInputHandler {
    /// Get the useable area of the textbox container
    fn update_dimensions(&mut self) {
        let (w, h) = size().unwrap_or((0, 0));
        self.y = h;
        self.x = w;
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

    fn write(&self, window: &mut MainWindow) -> Result<()> {
        // Remove the existing content
        window.reset_command_line()?;

        // Insert the word to the screen
        queue!(
            stdout(),
            cursor::MoveTo(1, self.y()),
            style::Print(self.get_content()),
            cursor::MoveTo(self.last_write, self.y()),
            cursor::Show
        )?;
        stdout().flush()?;
        Ok(())
    }

    /// Insert character to the input window
    /// TODO: Support insert vs normal typing mode
    fn insert_char(&mut self, window: &mut MainWindow, character: KeyCode) -> Result<()> {
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
                    self.write(window)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn position_as_index(&self) -> usize {
        (self.last_write - 1) as usize
    }

    /// Remove char 1 to the left of the cursor
    fn backspace(&mut self, window: &mut MainWindow) -> Result<()> {
        if self.last_write >= 1 && !self.content.is_empty() {
            self.content.remove(self.position_as_index() - 1);
            self.move_left()?;
            self.write(window)?;
        }
        Ok(())
    }

    /// Remove char 1 to the right of the cursor
    fn delete(&mut self, window: &mut MainWindow) -> Result<()> {
        if self.last_write < self.x() && !self.content.is_empty() {
            self.content.remove(self.position_as_index());
            self.write(window)?;
        }
        Ok(())
    }

    /// Move the cursor left
    fn move_left(&mut self) -> Result<()> {
        self.last_write = max(1, self.last_write.checked_sub(1).unwrap_or(1));
        queue!(stdout(), cursor::MoveTo(self.last_write, self.y()),)?;
        Ok(())
    }

    /// Move the cursor right
    fn move_right(&mut self) -> Result<()> {
        // TODO: possible index errors here
        self.last_write = min(self.content.len() as u16 + 1, self.last_write + 1);
        queue!(stdout(), cursor::MoveTo(self.last_write, self.y()))?;
        Ok(())
    }

    /// Get the next item in the history tape if it exists
    fn tape_forward(&mut self, window: &mut MainWindow) -> Result<()> {
        let content = self.history.scroll_forward();
        self.tape_render(window, &content)?;
        Ok(())
    }

    /// Get the previous item in the history tape if it exists
    fn tape_back(&mut self, window: &mut MainWindow) -> Result<()> {
        let content = self.history.scroll_back();
        self.tape_render(window, &content)?;
        Ok(())
    }

    /// Render the new choice
    fn tape_render(&mut self, window: &mut MainWindow, content: &str) -> Result<()> {
        self.last_write = content.len() as u16 + 1;
        window.write_to_command_line(content)?;
        self.content = content.chars().collect();
        queue!(
            stdout(),
            cursor::MoveTo(self.last_write, self.y()),
            cursor::Show
        )?;
        Ok(())
    }

    /// Get the contents of the command line as a String
    pub fn gather(&mut self, window: &mut MainWindow) -> Result<String> {
        // Copy the result to a new place so we can clear out the existing one and reuse the struct
        let result = self.get_content();
        self.content.clear();

        // Hide the cursor
        queue!(stdout(), cursor::Hide)?;

        // Reset the last written spot
        self.last_write = 1;
        window.reset_command_line()?;

        // Write to the history tape
        if window.config.use_history {
            match self.history.add_item(&result) {
                Ok(_) => {}
                Err(why) => window.write_to_command_line(&why.to_string())?,
            }
        }

        Ok(result)
    }
}

impl Handler for UserInputHandler {
    fn new() -> UserInputHandler {
        let mut handler = UserInputHandler {
            x: 0,
            y: 0,
            last_write: 1,
            content: vec![],
            history: Tape::new(),
        };
        handler.update_dimensions();
        handler
    }

    fn receive_input(&mut self, window: &mut MainWindow, key: KeyCode) -> Result<()> {
        queue!(stdout(), cursor::Show)?;
        match key {
            // Remove data
            KeyCode::Delete => self.delete(window)?,
            KeyCode::Backspace => self.backspace(window)?,

            // Move cursor
            // TODO: Possibly opt+left to skip words/symbols
            KeyCode::Left => self.move_left()?,
            KeyCode::Right => self.move_right()?,

            KeyCode::Up => self.tape_back(window)?,
            KeyCode::Down => self.tape_forward(window)?,

            // Insert char
            command => self.insert_char(window, command)?,
        }
        stdout().flush()?;
        Ok(())
    }
}
