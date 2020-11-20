pub mod main {
    use std::path::Path;
    use std::time::Instant;
    use std::cmp::max;

    use ncurses::{addstr, curs_set, getmaxyx, mv, wrefresh, CURSOR_VISIBILITY};

    use crate::communication::handlers::command::CommandHandler;
    use crate::communication::handlers::handler::HanderMethods;
    use crate::communication::handlers::multiple_choice::MultipleChoiceHandler;
    use crate::communication::handlers::normal::NormalHandler;
    use crate::communication::handlers::regex::RegexHandler;
    use crate::communication::handlers::parser::ParserHandler;
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
        messages: &'static Vec<String>, // Default to watching stderr
        previous_messages: Option<&'static Vec<String>>, // Pointer to the previous non-parsed message list, which is continuously updated
        exit_val: i8,                                    // If exit_val is -1, the app dies

        // Message buffers
        stderr_messages: Vec<String>,
        stdout_messages: Vec<String>,

        // Regex settings
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
        streams: Vec<InputStream>, // Can be a vector of FileInputs, CommandInputs, etc
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
                    height: 0,
                    width: 0,
                    loop_time: 0.0,
                    messages: &vec![], // Init to nothing
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
                    streams: streams,
                },
            }
        }

        fn determine_render_position(&mut self) -> (usize, usize) {            
            let mut end: usize = 0;
            let mut rows: usize = 0;
            let mut message_pointer_length: usize = 0;

            if self.config.stick_to_top {
                let current_index: usize = 0;
                loop {
                    let next_message: &str = match self.input_type {
                        InputType::Normal | InputType::MultipleChoice | InputType::Command => {
                            message_pointer_length = self.config.messages.len();
                            &self.config.messages[current_index]
                        },
                        InputType::Parser | InputType::Regex => {
                            message_pointer_length = self.config.matched_rows.len();
                            &self.config.messages[self.config.matched_rows[current_index]]
                        },
                    };

                    // Determine if we can fit the next message
                    let message_lines = next_message.len() / self.config.width as usize;
                    rows += message_lines;

                    // If we can fit, increment the last row number
                    if rows < self.config.last_row as usize && end < message_pointer_length {
                        end += 1;
                        continue;
                    }

                    // If the above if doesn't hit, we are done
                    break;
                }
            } else if self.config.stick_to_bottom {
                match self.config.matched_rows.len() {
                    0 => end = self.config.messages.len(),
                    other => end = other,
                }
            } else if self.config.manually_controlled_line {
                if message_pointer_length < self.config.last_row as usize {
                    // If have fewer messages than lines, just render it all
                    end = message_pointer_length - 1;
                } else if self.config.current_end < self.config.last_row as usize {
                    // If the last row we rendered comes before the last row we can render,
                    // use all of the available rows
                    end = self.config.current_end;
                } else if self.config.current_end < message_pointer_length {
                    // If we are looking at a valid line, render ends there
                    end = self.config.current_end;
                } else {
                    // If we have overscrolled, go back
                    if self.config.current_end > message_pointer_length {
                        self.config.current_end = message_pointer_length;
                    } else {
                        // Since current_end can be zero, we have to use the number of messages
                        end = message_pointer_length
                    }
                }
            } else {
                end = message_pointer_length
            }
            self.config.current_end = end; // Save this row so we know where we are
            let start = max(0, end - self.config.last_row as usize - 1);
            (start, end)
        }

        fn redraw(&self) {
            wrefresh(self.output());
        }

        pub fn screen(&self) -> ncurses::WINDOW {
            match self.stdscr {
                Some(scr) => scr,
                None => panic!("Attempted to get screen before screen has been initialized!"),
            }
        }

        pub fn output(&self) -> ncurses::WINDOW {
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

        pub fn go_to_cli(&self) {
            mv(self.config.height - 2, 1);
        }

        fn reset_command_line(&self) {
            // Leave padding for surrounding rectangle, we cannot use deleteln because it destroys the rectangle
            let clear = " ".repeat((self.config.width - 3) as usize); // TODO: Store this string as a class attribute, recalc on resize
            self.go_to_cli();
            addstr(&clear);

            // If the cursor was visible, hide it
            curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

            // Refresh the view
            wrefresh(self.input());
        }

        pub fn write_to_command_line(&self, content: &str) {
            // Remove what used to be in the command line
            self.reset_command_line();

            // Add the string to the front of the command line
            // TODO: Possibly validate length?
            self.go_to_cli();
            addstr(content);

            // If the cursor was visible, hide it
            curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

            // Refresh the view
            wrefresh(self.input());
        }

        pub fn start(&mut self) {
            // Build the UI, get reference to the text body content, etc
            self.stdscr = Some(init_scr());
            ncurses::nodelay(self.screen(), true);

            // Hide the cursor
            curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

            // Set dimensions
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

        /// Update stderr and stdout buffers from every stream's queue
        fn recieve_streams(&mut self) -> i32 {
            let mut total_messages = 0;
            for stream in &self.config.streams {
                // Read from streams until there is no more input
                // May lock if logs come in too fast
                loop {
                    match stream.stderr.try_recv() {
                        Ok(data) => {
                            total_messages += 1;
                            self.config.stderr_messages.push(data);
                        }
                        Err(_) => break,
                    }
                }
                loop {
                    match stream.stdout.try_recv() {
                        Ok(data) => {
                            total_messages += 1;
                            self.config.stdout_messages.push(data);
                        }
                        Err(_) => break,
                    }
                }
            }
            total_messages
        }

        fn main(&mut self) {
            // Main app loop
            let mut normal_handler = NormalHandler::new();
            let mut command_handler = CommandHandler::new();
            let mut regex_handler = RegexHandler::new();
            let mut parser_handler = ParserHandler::new();
            let mut mc_handler = MultipleChoiceHandler::new(); // Possibly different path for building options
                                                               // temp
            use crate::communication::handlers::user_input::UserInputHandler;
            let mut input_handler = UserInputHandler::new(); // input_handler.gather() to get contents

            // enum for input mode: {normal, command, regex, choice}
            // if input mode is command or regex, draw/remove the character to the command line
            // Otherwise, show status
            loop {
                // Update streams here
                let t_0 = Instant::now();
                let new_messages = self.recieve_streams();
                let t_1 = t_0.elapsed();
                println!("{} in {:?}", new_messages, t_1);

                match ncurses::getch() {
                    -1 => self.write_to_command_line("no input"), // possibly sleep
                    input => match self.input_type {
                        InputType::Normal => normal_handler.recieve_input(&self, input),
                        InputType::Command => command_handler.recieve_input(&self, input),
                        InputType::Regex => regex_handler.recieve_input(&self, input),
                        InputType::Parser => parser_handler.recieve_input(&self, input),
                        InputType::MultipleChoice => mc_handler.recieve_input(&self, input),
                    },
                }
                use std::{thread, time};
                let sleep = time::Duration::from_millis(500);
                thread::sleep(sleep);
            }
        }
    }
}
