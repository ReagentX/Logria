use crate::communication::reader::main::MainWindow;

// Used in Command and Regex handler to capture user typing
pub struct UserInputHandler {}

impl UserInputHandler {
    fn new() -> UserInputHandler {
        UserInputHandler{

        }
    }

    fn receive_input(&self, window: &MainWindow, key: i32) {
        window.write_to_command_line("got data in CommandHandler")
    }
}
