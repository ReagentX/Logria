use clap::{crate_version, App, Arg, ArgMatches};

use crate::constants::app::NAME;
use crate::constants::cli::messages;

pub fn from_command_line() -> ArgMatches {
    let matches = App::new(NAME)
        .version(crate_version!())
        .about(messages::APP_DESCRIPTION)
        .arg(
            Arg::new("history")
                .short('i')
                .long("no-history")
                .about(messages::HISTORY_HELP),
        )
        .arg(
            Arg::new("smart-poll-rate")
                .short('n')
                .long("no-smart-poll-rate")
                .about(messages::SMART_POLL_RATE_HELP),
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
