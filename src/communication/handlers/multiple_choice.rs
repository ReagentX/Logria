use std::collections::HashMap;

use crossterm::event::KeyCode;
use crossterm::Result;

use crate::{
    communication::{
        handlers::{handler::Handler, user_input::UserInputHandler},
        reader::MainWindow,
    },
    ui::scroll,
};

pub struct MultipleChoiceHandler {
    choices_map: HashMap<usize, String>,
    input_handler: UserInputHandler,
    pub result: Option<usize>,
}

impl MultipleChoiceHandler {
    /// Set internal choices map
    pub fn set_choices(&mut self, choices: &[String]) {
        self.choices_map.clear();
        choices.iter().enumerate().for_each(|(index, choice)| {
            self.choices_map.insert(index, choice.to_owned());
        })
    }

    /// Build body text for a set of choices
    pub fn get_body_text(&self) -> Vec<String> {
        let mut body_text: Vec<String> = vec![];
        (0..self.choices_map.len()).for_each(|key| {
            body_text.push(format!("{}: {}", key, self.choices_map.get(&key).unwrap()))
        });
        body_text
    }

    /// Determine if the choice is valid
    pub fn validate_choice(&mut self, window: &mut MainWindow, choice: &str) -> Result<()> {
        match choice.parse::<usize>() {
            Ok(res) => {
                if self.choices_map.contains_key(&res) {
                    self.result = Some(res.to_owned());
                } else {
                    window.write_to_command_line(&format!("Invalid item: {}", choice))?;
                }
            }
            Err(why) => {
                window
                    .write_to_command_line(&format!("Invalid selection: {} ({:?})", choice, why))?;
            }
        }
        Ok(())
    }

    /// Extract the choice value from the hashmap
    pub fn get_choice(&mut self) -> Option<&String> {
        match self.result {
            Some(index) => {
                self.result = None;
                self.choices_map.get(&index)
            }
            None => None,
        }
    }
}

impl Handler for MultipleChoiceHandler {
    fn new() -> MultipleChoiceHandler {
        MultipleChoiceHandler {
            choices_map: HashMap::new(),
            input_handler: UserInputHandler::new(),
            result: None,
        }
    }

    fn receive_input(&mut self, window: &mut MainWindow, key: KeyCode) -> Result<()> {
        match key {
            // Scroll
            KeyCode::Down => scroll::down(window),
            KeyCode::Up => scroll::up(window),
            KeyCode::Left => scroll::top(window),
            KeyCode::Right => scroll::bottom(window),
            KeyCode::Home => scroll::top(window),
            KeyCode::End => scroll::bottom(window),
            KeyCode::PageUp => scroll::pg_up(window),
            KeyCode::PageDown => scroll::pg_down(window),

            // Handle user input selection
            KeyCode::Enter => {
                let choice = match self.input_handler.gather(window) {
                    Ok(pattern) => pattern,
                    Err(why) => panic!("Unable to gather text: {:?}", why),
                };
                self.validate_choice(window, &choice)?;
                // Send 2 new refresh ticks from the main app loop when this method returns
                window.config.did_switch = true;
            }

            // User text input
            key => self.input_handler.receive_input(window, key)?,
        }
        window.redraw()?;
        Ok(())
    }
}

#[cfg(test)]
mod kc_tests {
    use std::collections::HashMap;

    use super::MultipleChoiceHandler;
    use crate::communication::{handlers::handler::Handler, reader::MainWindow};

    #[test]
    fn can_create() {
        MultipleChoiceHandler::new();
    }

    #[test]
    fn can_set_choices() {
        // Setup handler
        let mut mc = MultipleChoiceHandler::new();
        mc.set_choices(&["a".to_string(), "b".to_string(), "c".to_string()]);

        // Generate expected result
        let mut expected: HashMap<usize, String> = HashMap::new();
        expected.insert(0, String::from("a"));
        expected.insert(1, String::from("b"));
        expected.insert(2, String::from("c"));

        assert_eq!(mc.choices_map, expected);
    }

    #[test]
    fn can_get_body_text_no_desc() {
        // Setup handler
        let mut mc = MultipleChoiceHandler::new();
        mc.set_choices(&["a".to_string(), "b".to_string(), "c".to_string()]);

        // Generate expected result
        let expected = vec!["0: a", "1: b", "2: c"];

        assert_eq!(mc.get_body_text(), expected);
    }

    #[test]
    fn can_validate_choice() {
        // Setup Logria
        let mut logria = MainWindow::_new_dummy();

        // Setup handler
        let mut mc = MultipleChoiceHandler::new();
        mc.set_choices(&["a".to_string(), "b".to_string(), "c".to_string()]);

        // Generate expected result
        mc.validate_choice(&mut logria, "1").unwrap();

        assert_eq!(Some(1), mc.result);
    }

    #[test]
    fn can_get_choice() {
        // Setup Logria
        let mut logria = MainWindow::_new_dummy();

        // Setup handler
        let mut mc = MultipleChoiceHandler::new();
        mc.set_choices(&["a".to_string(), "b".to_string(), "c".to_string()]);

        // Generate expected result
        mc.validate_choice(&mut logria, "1").unwrap();

        assert_eq!("b", mc.get_choice().unwrap());
    }
}
