pub mod poll_rate {
    // Numerical limits
    // Fast enough for smooth typing, 1000Hz
    const FASTEST: f64 = 0.0001;
    // Poll ten times per second, 10Hz
    const SLOWEST: f64 = 0.1;
}

pub mod patterns {
    // TODO: this line need to exist
    // const ANSI_COLOR_PATTERN: &str = "(\x9B|\x1B\[)[0-?]*[ -\/]*[@-~]"
}

pub mod messages {
    // Text to exclude from message history
    const HISTORY_EXCLUDES: [&'static str; 2] = [
        ":history",
        ":history off"
    ];

    // Messages
    const START_MESSAGE: [&'static str; 7] = [
        "Enter a new command to open and save a new stream,",
        "or enter a number to choose a saved session from the list,",
        "or enter `:config` to configure.",
        " ",  // Blank line for printout
        "Enter `:r #` to remove session //.",
        "Enter `:q` to quit.",
        " "  // Not an empty string so Curses knows to not use this line
    ];

    // Config messages
    const CONFIG_START_MESSAGES: [&'static str; 2] = [
        "Saved data paths:",
        // f"Parsers:  {USER_HOME}/{SAVED_PATTERNS_PATH}",
        // f"Sessions: {USER_HOME}/{SAVED_SESSIONS_PATH}",
        "To configure new parameters, enter `session` or `parser`"
    ];
    const CREATE_SESSION_START_MESSAGES: [&'static str; 1] = [
        "To create a session, enter a type, either `command` or `file`:"
    ];
    const CREATE_PARSER_MESSAGES: [&'static str; 1] = [
        "To create a parser, enter a type, either `regex` or `split`:"
    ];

    // Session Strings
    const SESSION_ADD_COMMAND: &str = "Enter a command to open pipes to:";
    const SESSION_SHOULD_CONTINUE_COMMAND: &str = "Enter :s to save or press enter to add another command";
    const SESSION_ADD_FILE: &str = "Enter a path to a file:";
    const SESSION_SHOULD_CONTINUE_FILE: &str = "Enter :s to save or press enter to add another file";
    const SAVE_CURRENT_SESSION: &str = "Enter a name to save the session:";

    // Parser Strings
    const PARSER_SET_NAME: &str = "Enter a name for the parser:";
    const PARSER_SET_EXAMPLE: &str = "Enter an example string to match against:";
    const PARSER_SET_PATTERN: &str = "Enter a regex pattern:";
    const SAVE_CURRENT_PATTERN: &str = "Press enter to save or type `:q` to quit:";

    // Startup messages
    const APP_DESCRIPTION: &str = "A powerful CLI tool that puts log analytics at your fingertips.";
    const EXEC_HELP: &str = "Command to listen to, ex: logria -e \"tail -f log.txt\"";
    const HISTORY_HELP: &str = "Disable command history disk cache";
    const SMART_SPEED_HELP: &str = "Disable variable speed polling based on message receive rate";
    const PIPE_INPUT_ERROR: &str = "Piping is not supported as Logria cannot both
    listen to stdin as well as get user input from
    your tty. Process substitution is also not
    allowed, as Logria is unable to read from the
    file descriptor created by the shell.

    To capture command output, start Logria and
    enter the command during the setup process, or
    invoke Logria with `logria -e \"command\"";
}
