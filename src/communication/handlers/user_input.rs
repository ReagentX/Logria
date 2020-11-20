use crate::communication::reader::main::MainWindow;
use super::handler::HanderMethods;

// Used in Command and Regex handler to capture user typing
pub struct UserInputHandler {
    x: i32,
    y: i32
}

impl UserInputHandler {

    /// Get the useable area of the textbox container
    fn update_dimensions(&self, window: &MainWindow) {
        ncurses::getmaxyx(window.screen(), &mut self.y, &mut self.x)
    }

    fn validate(&self, key: &i32) -> i32 {
        match key {
            127 => 263,  // Ctrl-h to backspace
            key => *key,
        }
    }

    fn do_command(&self, window: &MainWindow, command: char) -> bool {

    }
    
}

impl HanderMethods for UserInputHandler {
    fn new() -> UserInputHandler {
        UserInputHandler{
            x: 0,
            y: 0,
        }
    }

    fn recieve_input(&self, window: &MainWindow, key: i32) {
        let valid_key = self.validate(&key);
        let character = self.get_char(valid_key);
        let success = self.do_command(window, character);
        window.write_to_command_line("got data in CommandHandler")
    }
}
