use crossterm::Result;
use crossterm::event::KeyCode;

use super::handler::HanderMethods;
use crate::communication::reader::main::MainWindow;

pub struct MultipleChoiceHandler {
    choices: Vec<&'static str>,
}

impl MultipleChoiceHandler {
    fn new_with_choices(choices: Vec<&'static str>) -> MultipleChoiceHandler {
        MultipleChoiceHandler { choices }
    }
}

impl HanderMethods for MultipleChoiceHandler {
    fn new() -> MultipleChoiceHandler {
        MultipleChoiceHandler { choices: vec![] }
    }

    fn recieve_input(&mut self, window: &mut MainWindow, key: KeyCode) -> Result<()> {
        window.write_to_command_line("got data in MultipleChoiceHandler")?;
        Ok(())
    }
}
