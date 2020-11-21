use crate::communication::reader::main::MainWindow;
use super::handler::HanderMethods;

pub struct CommandHandler {}

impl HanderMethods for CommandHandler {
    fn new() -> CommandHandler {
        CommandHandler {}
    }

    fn recieve_input(&mut self, window: &mut MainWindow, key: i32) {
        window.write_to_command_line("got data in CommandHandler")
    }
}
