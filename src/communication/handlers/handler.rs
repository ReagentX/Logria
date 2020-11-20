use crate::communication::reader::main::MainWindow;

pub struct Handler {}

pub trait HanderMethods {
    fn new() -> Self;
    fn get_char(&self, key: i32) -> char;
    fn recieve_input(&self, window: &MainWindow, key: i32);
}
