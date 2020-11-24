use crossterm::event::KeyCode;

use super::handler::HanderMethods;
use crate::communication::reader::main::MainWindow;

pub struct CommandHandler {}

impl HanderMethods for CommandHandler {
    fn new() -> CommandHandler {
        CommandHandler {}
    }

    fn recieve_input(&mut self, window: &mut MainWindow, key: KeyCode) {
        window.write_to_command_line(&format!("got data in CommandHandler: {:?}", key));
    }
}
