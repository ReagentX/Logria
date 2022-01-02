pub mod poll_rate {
    // Numerical limits in milliseconds
    // Fast enough for smooth typing, 1000Hz
    pub const FASTEST: u64 = 1;
    // Poll once per second, 1Hz
    pub const SLOWEST: u64 = 1000;
    // Default rate, 500 hz
    pub const DEFAULT: u64 = 50;
}

pub mod patterns {
    pub const ANSI_COLOR_PATTERN: &str = r"(?-u)(\x9b|\x1b\[)[0-?]*[ -/]*[@-~]";
}

pub mod colors {
    pub const RESET_COLOR: &str = "\x1b[0m";
    pub const HIGHLIGHT_COLOR: &str = "\x1b[35m";
}

pub mod excludes {
    // Text to exclude from message history
    pub const HISTORY_EXCLUDES: [&str; 2] = [":history", ":history off"];
    pub const SESSION_FILE_EXCLUDES: [&str; 1] = [".DS_Store"];
}

pub mod cli_chars {
    pub const NORMAL_CHAR: &str = "│";
    pub const COMMAND_CHAR: &str = ":";
    pub const REGEX_CHAR: &str = "/";
    pub const PARSER_CHAR: &str = "+";
}

pub mod messages {

    // Startup messages
    pub const START_MESSAGE: [&str; 6] = [
        "Enter a new command to open and save a new stream,",
        "or enter a number to choose a saved session from the list.",
        " ", // Blank line for printout
        "Enter `:r #` to remove session #.",
        "Enter `:q` to quit.",
        " ", // Blank line for printout
    ];

    // Error messages
    pub const NO_MESSAGE_IN_BUFFER_NORMAL: &str =
        "No messages in current buffer; press s to swap buffers.";
    pub const NO_MESSAGE_IN_BUFFER_PARSER: &str =
        "No messages match current parser rules. Press z to exit parsing mode.";

    // Config messages
    pub const CONFIG_START_MESSAGES: [&str; 2] = [
        "Saved data paths:",
        // f"Parsers:  {USER_HOME}/{SAVED_PATTERNS_PATH}",
        // f"Sessions: {USER_HOME}/{SAVED_SESSIONS_PATH}",
        "To configure new parameters, enter `session` or `parser`",
    ];
    pub const CREATE_SESSION_START_MESSAGES: [&str; 1] =
        ["To create a session, enter a type, either `command` or `file`:"];
    pub const CREATE_PARSER_MESSAGES: [&str; 1] =
        ["To create a parser, enter a type, either `regex` or `split`:"];

    // Session Strings
    pub const SESSION_ADD_COMMAND: &str = "Enter a command to open pipes to:";
    pub const SESSION_SHOULD_CONTINUE_COMMAND: &str =
        "Enter :s to save or press enter to add another command";
    pub const SESSION_ADD_FILE: &str = "Enter a path to a file:";
    pub const SESSION_SHOULD_CONTINUE_FILE: &str =
        "Enter :s to save or press enter to add another file";
    pub const SAVE_CURRENT_SESSION: &str = "Enter a name to save the session:";

    // Parser Strings
    pub const PARSER_SET_NAME: &str = "Enter a name for the parser:";
    pub const PARSER_SET_EXAMPLE: &str = "Enter an example string to match against:";
    pub const PARSER_SET_PATTERN: &str = "Enter a regex pattern:";
    pub const SAVE_CURRENT_PATTERN: &str = "Press enter to save or type `:q` to quit:";

    // Startup messages
    pub const APP_DESCRIPTION: &str =
        "A powerful CLI tool that puts log analytics at your fingertips.";
    pub const EXEC_HELP: &str = "Command to listen to, ex: logria -e \"tail -f log.txt\"";
    pub const HISTORY_HELP: &str = "Disable command history disk cache";
    pub const SMART_POLL_RATE_HELP: &str =
        "Disable variable polling rate based on incoming message rate";
    pub const DOCS_HELP: &str = "Prints documentation";
    pub const DOCS: &str = concat!(
        "CONTROLS:\n",
        "    +------+--------------------------------------------------+\n",
        "    | Key  | Command                                          |\n",
        "    +======+==================================================+\n",
        "    |  :   | command mode                                     |\n",
        "    |  /   | regex search                                     |\n",
        "    |  h   | if regex active, toggle highlighting of matches  |\n",
        "    |  s   | swap reading `stderr` and `stdout`               |\n",
        "    |  p   | activate parser                                  |\n",
        "    |  a   | toggle aggregation mode when parser is active    |\n",
        "    |  z   | deactivate parser                                |\n",
        "    |  ↑   | scroll buffer up one line                        |\n",
        "    |  ↓   | scroll buffer down one line                      |\n",
        "    |  →   | skip and stick to end of buffer                  |\n",
        "    |  ←   | skip and stick to beginning of buffer            |\n",
        "    +------+--------------------------------------------------+\n\n",
        "COMMANDS:\n",
        "    +-----------------+---------------------------------------+\n",
        "    | Key             | Command                               |\n",
        "    +=================+=======================================+\n",
        "    | :q              | exit Logria                           |\n",
        "    | :poll #         | update poll rate to #, where # is an  |\n",
        "    |                 | integer (in milliseconds)             |\n",
        "    | :r #            | when launching logria or viewing      |\n",
        "    |                 | sessions, this will delete item #     |\n",
        "    | :restart        | go back to the setup screen to change |\n",
        "    |                 | sessions, this will delete item #     |\n",
        "    | :agg #          | set the limit for aggregation counters|\n",
        "    |                 | be top #, i.e. top 5 or top 1         |\n",
        "    +-----------------+---------------------------------------|\n"
    );
    pub const PIPE_INPUT_ERROR: &str = concat!(
        "Piping is not supported as Logria cannot both\n",
        "listen to stdin as well as get user input from\n",
        "your tty. Process substitution is also not\n",
        "allowed, as Logria is unable to read from the\n",
        "file descriptor created by the shell.\n",
        "\n",
        "To capture command output, start Logria and\n",
        "enter the command during the setup process,\n",
        "invoke Logria with `logria -e \"command\", or",
        "create a valid session file."
    );
}
