#![forbid(unsafe_code)]

use crossterm::Result;

mod communication;
mod constants;
mod extensions;
mod ui;
mod util;

use communication::reader::MainWindow;
use constants::{cli::messages::DOCS, directories::print_paths};
use util::options::from_command_line;

fn main() -> Result<()> {
    // Get options from command line
    let options = from_command_line();
    if options.get_flag("docs") {
        println!("{}", DOCS);
    } else if options.get_flag("paths") {
        print_paths();
    } else {
        let history = !options.get_flag("history");
        let smart_poll_rate = !options.get_flag("mindless");
        let exec: Option<Vec<String>> = match options.try_get_one("exec") {
            Ok(cmd) => cmd.map(|text: &String| vec![text.to_string()]),
            Err(_) => None,
        };

        // Start app
        let mut app = MainWindow::new(history, smart_poll_rate);
        app.start(exec)?;
    }
    Ok(())
}
