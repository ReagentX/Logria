use crate::communication::reader::main::MainWindow;
use super::handler::HanderMethods;

pub struct MultipleChoiceHandler {
    choices: Vec<&'static str>,
}

impl MultipleChoiceHandler {
    fn new_with_choices(choices: Vec<&'static str>) -> MultipleChoiceHandler {
        MultipleChoiceHandler {
            choices: choices
        }
    }
}

impl HanderMethods for MultipleChoiceHandler {
    fn new() -> MultipleChoiceHandler {
        MultipleChoiceHandler {
            choices: vec![]
        }
    }

    fn recieve_input(&self, window: &MainWindow, key: i32) {
        window.write_to_command_line("got data in MultipleChoiceHandler")
    }
}
