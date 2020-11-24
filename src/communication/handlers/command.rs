use std::io::Write;

use crossterm::event::KeyCode;

use super::handler::HanderMethods;
use crate::communication::handlers::user_input::UserInputHandler;
use crate::communication::input::input_type::InputType::Normal;
use crate::communication::reader::main::MainWindow;

pub struct CommandHandler {
    input_hander: UserInputHandler,
}

impl CommandHandler {
    fn return_to_prev_state(&mut self, window: &mut MainWindow) {
        window.input_type = Normal;
        window.set_cli_cursor(None);
        window.output.flush();
    }

    fn process_command(&mut self, window: &MainWindow, command: &str) {
        match command {
            "q" => {}
            "poll" => {}
            "config" => {}
            "history" => {}
            "history off" => {}
            "r" => {}
            "restart" => {}
            _ => {}
        }
    }
}

impl HanderMethods for CommandHandler {
    fn new() -> CommandHandler {
        CommandHandler {
            input_hander: UserInputHandler::new(),
        }
    }

    fn recieve_input(&mut self, window: &mut MainWindow, key: KeyCode) {
        match key {
            // Execute the command
            KeyCode::Enter => {
                let command = self.input_hander.gather(window);
                self.process_command(window, &command);
            }
            // Go back to the previous state
            KeyCode::Esc => self.return_to_prev_state(window),
            key => self.input_hander.recieve_input(window, key),
        }
    }
}
