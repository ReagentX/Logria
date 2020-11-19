pub mod main {
    use ncurses::{curs_set, getmaxyx, mvaddstr, wrefresh, CURSOR_VISIBILITY};
    use std::path::Path;

    use crate::communication::input::input_type::InputType;
    use crate::communication::input::stream::{FileInput, InputStream};
    use crate::constants::cli::poll_rate::FASTEST;
    use crate::ui::interface::build::{command_line, exit_scr, init_scr, output_window};

    #[derive(Debug)]
    pub struct LogiraConfig {
        pub poll_rate: u64,    // The rate at which we check for new messages
        pub height: i32,       // Window height
        pub width: i32,        // Window width
        pub last_row: i32,     // The last row we can render, aka number of lines visible in the tty
        smart_poll_rate: bool, // Whether we reduce the poll rate to the message receive speed
        first_run: bool,       // Whether this is a first run or not
        loop_time: f64,        // How long a loop of the main app takes
        messages: Option<&'static Vec<String>>, // Default to watching stderr
        previous_messages: Option<&'static Vec<String>>, // Pointer to the previous non-parsed message list, which is continuously updated
        exit_val: i8,                                    // If exit_val is -1, the app dies

        // Message buffers
        stderr_messages: Vec<String>,
        stdout_messages: Vec<String>,

        // Regex settings
        // func_handle: // Reference to current regex test func
        regex_pattern: Option<String>, // Current regex pattern
        matched_rows: Vec<usize>,      // List of index of matches when regex filtering is active
        last_index_regexed: usize,     // The last index the filtering function saw

        // Parser settings
        // parser: ???  // Reference to the current parser
        parser_index: usize,            // Index for the parser to look at
        parsed_messages: Vec<String>,   // List of parsed messages
        analytics_enabled: bool,        // Whetehr we are calcualting stats or not
        last_index_processed: usize,    // The last index the parsing function saw
        insert_mode: bool,              // Default to insert mode (like vim) off
        current_status: String,         // Current status, aka what is in the command line
        highlight_match: bool,          // Determines whether we highlight the match to the user
        stick_to_bottom: bool,          // Whether we should follow the stream
        stick_to_top: bool, // Whether we should stick to the top and not render new lines
        manually_controlled_line: bool, // Whether manual scroll is active
        current_end: usize, // Current last row we have rendered
        stream: Vec<InputStream>, // Can be a vector of FileInputs, CommandInputs, etc
    }

    pub struct MainWindow {
        pub config: LogiraConfig,
        input_type: InputType,
        stdscr: Option<ncurses::WINDOW>,
        output: Option<ncurses::WINDOW>, // fix
        input: Option<ncurses::WINDOW>,  // fix
    }

    impl MainWindow {
        fn build_streams(commands: Vec<String>) -> Vec<InputStream> {
            let mut streams: Vec<InputStream> = vec![];
            for command in commands {
                // determine if command is a file, create FileInput
                // otherwise, create CommandInput
                let name = Path::new(&command)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                streams.push(FileInput::new(None, name, command)); // None indicates default poll rate
            }
            streams
        }

        pub fn new(cache: bool, smart_poll_rate: bool, commands: Vec<String>) -> MainWindow {
            // Build streams here
            let streams = MainWindow::build_streams(commands);
            MainWindow {
                stdscr: None,
                input_type: InputType::Normal,
                output: None,
                input: None,
                config: LogiraConfig {
                    poll_rate: FASTEST,
                    smart_poll_rate: smart_poll_rate,
                    first_run: true,
                    height: 0, // fix
                    width: 0,  // fix
                    loop_time: 0.0,
                    messages: None,
                    previous_messages: None,
                    exit_val: 0,
                    stderr_messages: vec![], // fix
                    stdout_messages: vec![], // fix
                    regex_pattern: None,
                    matched_rows: vec![],
                    last_index_regexed: 0,
                    parser_index: 0,
                    parsed_messages: vec![],
                    analytics_enabled: false,
                    last_index_processed: 0,
                    insert_mode: false,
                    current_status: String::from(""), // fix
                    highlight_match: false,
                    last_row: 0,
                    stick_to_bottom: true,
                    stick_to_top: false,
                    manually_controlled_line: false,
                    current_end: 0,
                    stream: streams,
                },
            }
        }

        fn redraw(&self) {
            wrefresh(self.output());
        }

        fn screen(&self) -> ncurses::WINDOW {
            match self.stdscr {
                Some(scr) => scr,
                None => panic!("Attempted to get screen before screen has been initialized!"),
            }
        }

        fn output(&self) -> ncurses::WINDOW {
            match self.output {
                Some(scr) => scr,
                None => panic!(
                    "Attempted to get output window before output window has been initialized!"
                ),
            }
        }

        fn input(&self) -> ncurses::WINDOW {
            match self.input {
                Some(scr) => scr,
                None => panic!(
                    "Attempted to get command line before command line has been initialized!"
                ),
            }
        }

        fn reset_command_line(&self) {
            // Leave padding for surrounding rectangle, we cannot use deleteln because it destroys the rectangle
            let clear = " ".repeat((self.config.width - 3) as usize);
            mvaddstr(self.config.height - 2, 1, &clear);

            // If the cursor was visible, hide it
            curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

            // Refresh the view
            wrefresh(self.input());
        }

        fn write_to_command_line(&self, content: &str) {
            // Remove what used to be in the command line
            self.reset_command_line();

            // Add the string to the front of the command line
            // TODO: Possibly validate length?
            mvaddstr(self.config.height - 2, 1, content);

            // If the cursor was visible, hide it
            curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

            // Refresh the view
            wrefresh(self.input());
        }

        pub fn start(&mut self) {
            // Build the UI, get reference to the text body content, etc
            self.stdscr = Some(init_scr());
            ncurses::nodelay(self.screen(), true);

            // This is unsafe
            getmaxyx(
                self.screen(),
                &mut self.config.height,
                &mut self.config.width,
            );
            self.config.last_row = self.config.height - 3;

            // Build output window
            self.output = Some(output_window(&self.config));

            // Build command line
            self.input = Some(command_line(self.screen(), &self.config));
            // Start the main event loop
            self.main();
        }

        fn main(&mut self) {
            // Main app loop

            // enum for input mode: {normal, command, regex, choice}
            // if input mode is command or regex, draw/remove the character to the command line
            // Otherwise, show status
            loop {
                match ncurses::getch() {
                    -1 => self.write_to_command_line("no input"), // possibly sleep
                    input => match self.input_type {
                        InputType::Normal => self.write_to_command_line("normal"),
                        InputType::Command => self.write_to_command_line("command"),
                        InputType::Regex => self.write_to_command_line("regex"),
                        InputType::MultipleChoice => self.write_to_command_line("mc"),
                    },
                }
                use std::{thread, time};
                let sleep = time::Duration::from_millis(500);
                thread::sleep(sleep);
            }
        }
    }
}
