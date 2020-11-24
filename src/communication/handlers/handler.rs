use crossterm::event::KeyCode;

use crate::communication::reader::main::MainWindow;

pub struct Handler {}

pub trait HanderMethods {
    fn new() -> Self;
    fn recieve_input(&mut self, window: &mut MainWindow, key: KeyCode);
}
