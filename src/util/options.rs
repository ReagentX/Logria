use clap::{crate_version, App, Arg, ArgMatches};

use crate::strings::app::NAME;
use crate::strings::cli::messages;

pub fn from_command_line() -> ArgMatches {
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
    return matches;
}
