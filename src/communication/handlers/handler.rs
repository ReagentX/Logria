use crate::communication::reader::main::MainWindow;

pub struct Handler {}

pub trait HanderMethods {
    fn new() -> Self;
    fn recieve_input(&mut self, window: &MainWindow, key: i32);
    
    fn get_char(&self, key: u32) -> char {
        match std::char::from_u32(key) {
            Some(character) => character,
            None => panic!("Invalid char typed!")
        }
    }
}
