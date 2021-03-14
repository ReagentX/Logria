use crossterm::Result;

mod communication;
mod constants;
mod extensions;
mod ui;
mod util;
use std::sync::Arc;

fn main() -> Result<()> {
    let options = util::options::from_command_line();
    let history = !options.is_present("history");
    let smart_poll_rate = options.is_present("smart-poll-rate");
    let exec: Option<Vec<String>> = match options.value_of("exec"){
        Some(text) => Some(vec![text.to_string()]),
        None => None,
    };

    // loop {
    //     let poll_rate = Arc::clone(&input.poll_rate);
    //     *poll_rate.lock().unwrap() += 100;
    //     println!("{:?}", poll_rate);
    //     println!("got data: {:?}", input.stderr.recv().unwrap());
    // }
        
    // Start app
    let mut app = communication::reader::main::MainWindow::new(history, smart_poll_rate);
    // app.start(
    //     Some(vec![exec.unwrap_or("python3 .logria/sample_streams/generate_test_logs_2.py").to_string()])
    // )?;
    app.start(exec)?;
    Ok(())
}
