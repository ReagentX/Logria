use crossterm::Result;
use crossterm::event::KeyCode;

use crate::communication::reader::main::MainWindow;

pub trait Handler {
    fn new() -> Self;
    fn recieve_input(&mut self, window: &mut MainWindow, key: KeyCode) -> Result<()>;
}
