use clap::{crate_version, App, Arg, ArgMatches};

use crate::constants::app::NAME;
use crate::constants::cli::messages;

pub fn from_command_line() -> ArgMatches<'static> {
    let matches = App::new(NAME)
        .version(crate_version!())
        .about(messages::APP_DESCRIPTION)
        .arg(
            Arg::with_name("history")
                .short("i")
                .long("no-history")
                .help(messages::HISTORY_HELP),
        )
        .arg(
            Arg::with_name("smart-poll-rate")
                .short("n")
                .long("no-smart-poll-rate")
                .help(messages::SMART_POLL_RATE_HELP),
        )
        .arg(
            Arg::with_name("exec")
                .short("e")
                .long("exec")
                .help(messages::EXEC_HELP)
                .takes_value(true)
                .value_name("stream"),
        )
        .get_matches();
    matches
}
