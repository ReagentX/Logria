use crossterm::Result;

use crate::communication::reader::main::MainWindow;

pub trait ProcessorMethods {
    fn return_to_normal(&mut self, window: &mut MainWindow) -> Result<()>;
    fn clear_matches(&mut self, window: &mut MainWindow) -> Result<()>;
    fn process_matches(&self, window: &mut MainWindow);
}
