use std::cmp::min;
use std::error::Error;
use std::fs::{OpenOptions, create_dir_all};
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
    // Ensure the proper paths exist
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
                self.current_index = self.current_index.checked_sub(num_to_scroll).unwrap_or_default();
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
