pub mod strings;
use clap::{crate_version, App, Arg};

use strings::app::NAME;
use strings::cli::messages;

fn main() {
    let matches = App::new(NAME)
        .version(crate_version!())
        .about(messages::APP_DESCRIPTION)
        .arg(
            Arg::new("cache")
                .short('c')
                .long("no-cache")
                .about(messages::HISTORY_HELP),
        )
        .arg(
            Arg::new("smart-speed")
                .short('n')
                .long("no-smart-speed")
                .about(messages::SMART_SPEED_HELP),
        )
        .arg(
            Arg::new("exec")
                .short('e')
                .long("exec")
                .about(messages::EXEC_HELP)
                .takes_value(true)
                .value_name("stream"),
        )
        .get_matches();

    let cache = matches.is_present("cache");
    let smart_speed = matches.is_present("smart-speed");
    let exec = matches.value_of("exec");
    println!("history disabled? {:?}", cache);
    println!("smart speed disabled? {:?}", smart_speed);
    println!("exec stream? {:?}", exec);
}
