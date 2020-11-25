use crossterm::Result;
use crossterm::event::KeyCode;

use super::handler::HanderMethods;
use crate::communication::reader::main::MainWindow;

pub struct ParserHandler {}

impl HanderMethods for ParserHandler {
    fn new() -> ParserHandler {
        ParserHandler {}
    }

    fn recieve_input(&mut self, window: &mut MainWindow, key: KeyCode) -> Result<()> {
        window.write_to_command_line("got data in ParserHandler")?;
        Ok(())
    }
}
