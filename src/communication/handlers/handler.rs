use crate::communication::reader::main::MainWindow;

pub struct Handler {}

pub trait HanderMethods {
    fn new() -> Self;
    fn recieve_input(&self, window: &MainWindow, key: i32);
}