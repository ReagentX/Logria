use std::convert::TryInto;

use crate::communication::reader::main::MainWindow;

pub struct Handler {}

pub trait HanderMethods {
    fn new() -> Self;
    fn recieve_input(&self, window: &MainWindow, key: i32);
    
    fn get_char(&self, key: i32) -> char {
        match std::char::from_u32(key.try_into().unwrap()) {
            Some(character) => character,
            None => panic!("Invalid char typed!")
        }
    }
}
