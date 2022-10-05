use clap::{command, crate_version, Arg, ArgAction, ArgMatches};

use crate::constants::app::NAME;
use crate::constants::cli::messages;

pub fn from_command_line() -> ArgMatches {
    let matches = command!(NAME)
        .version(crate_version!())
        .about(messages::APP_DESCRIPTION)
        .arg(
            Arg::new("history")
                .short('h')
                .long("no-history")
                .required(false)
                .action(ArgAction::SetTrue)
                .help(messages::HISTORY_HELP),
        )
        .arg(
            Arg::new("mindless")
                .short('m')
                .long("mindless")
                .required(false)
                .action(ArgAction::SetTrue)
                .help(messages::SMART_POLL_RATE_HELP),
        )
        .arg(
            Arg::new("docs")
                .short('d')
                .long("docs")
                .required(false)
                .action(ArgAction::SetTrue)
                .help(messages::DOCS_HELP),
        )
        .arg(
            Arg::new("paths")
                .short('p')
                .long("paths")
                .required(false)
                .action(ArgAction::SetTrue)
                .help(messages::PATHS_HELP),
        )
        .arg(
            Arg::new("exec")
                .short('e')
                .long("exec")
                .help(messages::EXEC_HELP)
                .value_name("stream"),
        )
        .get_matches();
    matches
}
