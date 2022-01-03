use crossterm::Result;
use crossterm::event::KeyCode;

use crate::communication::reader::MainWindow;

pub trait Handler {
    fn new() -> Self;
    fn receive_input(&mut self, window: &mut MainWindow, key: KeyCode) -> Result<()>;
}
