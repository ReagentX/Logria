#![allow(dead_code)] // REMOVE!!!

mod communication;
mod constants;
mod ui;
mod util;
use std::sync::Arc;

fn main() {
    let options = util::options::from_command_line();
    let cache = options.is_present("cache");
    let smart_poll_rate = options.is_present("smart-poll-rate");
    let exec = options.value_of("exec");

    // Build ui
    // loop {
    //     let poll_rate = Arc::clone(&input.poll_rate);
    //     *poll_rate.lock().unwrap() += 100;
    //     println!("{:?}", poll_rate);
    //     println!("got data: {:?}", input.stderr.recv().unwrap());
    // }

    let mut app = communication::reader::main::MainWindow::new(
        cache,
        smart_poll_rate,
    );
    app.start(vec![exec.unwrap_or("Cargo.toml").to_string()]);
}
