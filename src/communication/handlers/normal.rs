use crate::communication::reader::main::MainWindow;
use super::handler::HanderMethods;

pub struct NormalHandler {}

impl HanderMethods for NormalHandler {
    fn new() -> NormalHandler {
        NormalHandler {}
    }

    fn recieve_input(&self, window: &MainWindow, key: i32) {
        window.write_to_command_line("got data in NormalHandler")
    }
}
