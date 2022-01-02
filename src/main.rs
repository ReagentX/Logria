use crossterm::Result;

mod communication;
mod constants;
mod extensions;
mod ui;
mod util;

use communication::reader::main::MainWindow;
use constants::cli::messages::DOCS;
use util::options::from_command_line;

fn main() -> Result<()> {
    // Get options from command line
    let options = from_command_line();
    if options.is_present("docs") {
        println!("{}", DOCS);
    } else {
        let history = !options.is_present("history");
        let smart_poll_rate = !options.is_present("smart-poll-rate");
        let exec: Option<Vec<String>> = options.value_of("exec").map(|text| vec![text.to_string()]);

        // Start app
        let mut app = MainWindow::new(history, smart_poll_rate);
        app.start(exec)?;
    }
    Ok(())
}
