use crossterm::Result;

use crate::communication::reader::MainWindow;

pub trait ProcessorMethods {
    fn return_to_normal(&mut self, window: &mut MainWindow) -> Result<()>;
    fn clear_matches(&mut self, window: &mut MainWindow) -> Result<()>;
    fn process_matches(&mut self, window: &mut MainWindow) -> Result<()>;
}
