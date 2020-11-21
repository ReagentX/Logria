use super::handler::HanderMethods;
use crate::communication::reader::main::MainWindow;

pub struct NormalHandler {}

impl NormalHandler {
    fn scroll_up(&self, window: &mut MainWindow) {}

    fn scroll_down(&self, window: &mut MainWindow) {}

    fn pg_up(&self, window: &mut MainWindow) {}

    fn pg_down(&self, window: &mut MainWindow) {}

    fn bottom(&self, window: &mut MainWindow) {}

    fn top(&self, window: &mut MainWindow) {}

    fn noop(&self) {}
}

impl HanderMethods for NormalHandler {
    fn new() -> NormalHandler {
        NormalHandler {}
    }

    fn recieve_input(&mut self, window: &mut MainWindow, key: i32) {
        window.write_to_command_line(&format!("got data in NormalHandler: {}", key));
        match key {
            258 => self.scroll_down(&mut window), // down
            259 => self.scroll_up(&mut window),   // up
            260 => self.top(&mut window),         // left
            261 => self.bottom(&mut window),      // right
            262 => self.top(&mut window),         // home
            263 => self.bottom(&mut window),      // end
            338 => self.pg_down(&mut window),     // pgdn
            339 => self.pg_up(&mut window),       // pgup
            _ => self.noop(),
        }
    }
}
