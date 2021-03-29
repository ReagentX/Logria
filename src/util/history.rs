use std::cmp::min;
use std::error::Error;
use std::fs::{create_dir_all, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::constants::cli::excludes::HISTORY_EXCLUDES;
use crate::constants::directories::history_tape;

pub struct Tape {
    history_tape: Vec<String>,
    current_index: usize,
    should_scroll_back: bool,
}

impl Tape {
    /// Ensure the proper paths exist
    pub fn verify_path() {
        let tape_path = history_tape();
        if !Path::new(&tape_path).exists() {
            create_dir_all(tape_path).unwrap();
        }
    }

    pub fn new() -> Tape {
        Tape::verify_path();
        let mut tape = Tape {
            history_tape: vec![],
            current_index: 0,
            should_scroll_back: false,
        };
        tape.read_from_disk();
        tape
    }

    /// Read the history file from the disk to the current history buffer
    fn read_from_disk(&mut self) {
        let file = match OpenOptions::new().read(true).open(history_tape()) {
            // The `description` method of `io::Error` returns a string that describes the error
            Err(why) => panic!(
                "Couldn't open {:?}: {}",
                history_tape(),
                Error::to_string(&why)
            ),
            Ok(file) => file,
        };

        // Create a buffer and read from it
        let reader = BufReader::new(file);
        for line in reader.lines() {
            if line.is_ok() {
                self.history_tape.push(match line {
                    Ok(a) => a,
                    _ => unreachable!(),
                });
            }
        }

        self.current_index = self.history_tape.len() - 1;
    }

    /// Add an item to the history tape
    pub fn add_item(&mut self, item: &str) {
        let clean_item = item.trim();
        if !HISTORY_EXCLUDES.contains(&clean_item) {
            // Write to internal buffer
            self.history_tape.push(String::from(clean_item));

            // Reset tape to end
            self.should_scroll_back = false;
            self.current_index = self.history_tape.len() - 1;

            // Write to file
            let mut file = match OpenOptions::new()
                .read(true)
                .append(true)
                .open(history_tape())
            {
                // The `description` method of `io::Error` returns a string that describes the error
                Err(why) => panic!(
                    "Couldn't open {:?}: {}",
                    history_tape(),
                    Error::to_string(&why)
                ),
                Ok(file) => file,
            };
            match writeln!(file, "{}", clean_item) {
                Ok(_) => {}
                Err(why) => panic!(
                    "Couldn't write to {:?}: {}",
                    history_tape(),
                    Error::to_string(&why)
                ),
            }
        }
    }

    /// Rewind the tape if possible
    fn scroll_back_n(&mut self, num_to_scroll: usize) {
        if !self.history_tape.is_empty() {
            if self.should_scroll_back {
                self.current_index = self
                    .current_index
                    .checked_sub(num_to_scroll)
                    .unwrap_or_default();
            } else {
                self.should_scroll_back = true
            }
        }
    }

    /// Scroll the tape forward if possible
    fn scroll_forward_n(&mut self, num_to_scroll: usize) {
        if self.current_index != self.history_tape.len() - 1 && !self.history_tape.is_empty() {
            self.current_index = min(
                self.history_tape.len() - 1,
                self.current_index
                    .checked_add(num_to_scroll)
                    .unwrap_or(self.history_tape.len() - 1),
            );
        }
    }

    /// Common case where we scroll back a single item
    pub fn scroll_back(&mut self) -> String {
        self.scroll_back_n(1);
        self.get_current_item()
    }

    /// Common case where we scroll up a single item
    pub fn scroll_forward(&mut self) -> String {
        self.scroll_forward_n(1);
        self.get_current_item()
    }

    pub fn get_current_item(&self) -> String {
        self.history_tape[self.current_index].clone()
    }
}

#[cfg(test)]
mod tests {
    use super::Tape;

    #[test]
    fn can_construct() {
        Tape::new();
    }

    #[test]
    fn can_add_item() {
        let mut tape = Tape::new();
        tape.add_item("test");
        assert_eq!(String::from("test"), tape.get_current_item());
    }

    #[test]
    fn scroll_back_n_good() {
        let mut tape = Tape::new();
        tape.should_scroll_back = true;
        tape.scroll_back_n(5);
        assert_eq!(tape.current_index, tape.history_tape.len() - 5 - 1)
    }

    #[test]
    fn scroll_back_n_too_many() {
        let mut tape = Tape::new();
        tape.should_scroll_back = true;
        tape.scroll_back_n(tape.history_tape.len() * 2);
        assert_eq!(tape.current_index, 0)
    }

    #[test]
    fn scroll_back_one() {
        let mut tape = Tape::new();
        tape.should_scroll_back = true;
        tape.scroll_back();
        assert_eq!(tape.current_index, tape.history_tape.len() - 1 - 1)
    }

    #[test]
    fn scroll_forward_n_good() {
        let mut tape = Tape::new();
        tape.should_scroll_back = true;
        tape.scroll_back_n(10);
        tape.scroll_forward_n(5);
        assert_eq!(tape.current_index, tape.history_tape.len() - 5 - 1)
    }

    #[test]
    fn scroll_forward_n_too_many() {
        let mut tape = Tape::new();
        tape.should_scroll_back = true;
        tape.scroll_back_n(10);
        tape.scroll_forward_n(25);
        assert_eq!(tape.current_index, tape.history_tape.len() - 1)
    }

    #[test]
    fn scroll_forward_one() {
        let mut tape = Tape::new();
        tape.should_scroll_back = true;
        tape.scroll_back();
        tape.scroll_forward();
        assert_eq!(tape.current_index, tape.history_tape.len() - 1)
    }
}
