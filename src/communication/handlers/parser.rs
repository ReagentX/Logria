use crate::communication::reader::main::MainWindow;
use super::handler::HanderMethods;

pub struct ParserHandler {}

impl HanderMethods for ParserHandler {
    fn new() -> ParserHandler {
        ParserHandler {}
    }

    fn recieve_input(&mut self, window: &mut MainWindow, key: i32) {
        window.write_to_command_line("got data in ParserHandler")
    }
}
