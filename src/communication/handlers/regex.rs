use crate::communication::reader::main::MainWindow;
use super::handler::HanderMethods;

pub struct RegexHandler {}

impl HanderMethods for RegexHandler {
    fn new() -> RegexHandler {
        RegexHandler {}
    }

    fn recieve_input(&self, window: &MainWindow, key: i32) {
        window.write_to_command_line("got data in RegexHandler")
    }
}
