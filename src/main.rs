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
    if options.is_present("docs") {
        println!("{}", DOCS);
    } else if options.is_present("paths") {
        print_paths();
    } else {
        let history = !options.is_present("history");
        let smart_poll_rate = !options.is_present("mindless");
        let exec: Option<Vec<String>> = options.value_of("exec").map(|text| vec![text.to_string()]);

        // Start app
        // TODO: App should be stored like Arc<Mutex<App>> and shared between threads
        // ? Compute happens in the command subprocesses
        // ? Subprocesses can own the stdin and stdout vectors
        // ? To know if we neeed to render we can also share new messages memory
        // ? Input maybe in Tokio event loop? Async may be overhead
        let mut app = MainWindow::new(history, smart_poll_rate);
        app.start(exec)?;
        // Start the main event loop
        MainWindow::main(app)?;
    }
    Ok(())
}
