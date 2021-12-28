use crossterm::Result;

mod communication;
mod constants;
mod extensions;
mod ui;
mod util;

fn main() -> Result<()> {
    // Get options from command line
    let options = util::options::from_command_line();
    let history = !options.is_present("history");
    let smart_poll_rate = !options.is_present("smart-poll-rate");
    let exec: Option<Vec<String>> = options.value_of("exec").map(|text| vec![text.to_string()]);

    // Start app
    let mut app = communication::reader::main::MainWindow::new(history, smart_poll_rate);
    app.start(exec)?;
    Ok(())
}
