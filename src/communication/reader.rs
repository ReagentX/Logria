pub mod main {
    use std::cmp::{max, min};
    use std::path::Path;
    use std::time::Instant;

    use ncurses::{addstr, curs_set, getmaxyx, mv, mvwaddstr, mvwaddch, wrefresh, CURSOR_VISIBILITY};

    use crate::communication::handlers::command::CommandHandler;
    use crate::communication::handlers::handler::HanderMethods;
    use crate::communication::handlers::multiple_choice::MultipleChoiceHandler;
    use crate::communication::handlers::normal::NormalHandler;
    use crate::communication::handlers::parser::ParserHandler;
    use crate::communication::handlers::regex::RegexHandler;
    use crate::communication::input::input_type::InputType;
    use crate::communication::input::stream::{FileInput, InputStream};
    use crate::communication::input::stream_type::StreamType;
    use crate::constants::cli::poll_rate::FASTEST;
    use crate::constants::cli::cli_chars;
    use crate::util::sanitizers::length::LengthFinder;
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
        previous_render: (usize, usize),
        previous_messages: Option<&'static Vec<String>>, // Pointer to the previous non-parsed message list, which is continuously updated
        exit_val: i8,                                    // If exit_val is -1, the app dies

        // Message buffers
        stderr_messages: Vec<String>,
        stdout_messages: Vec<String>,
        pub stream_type: StreamType,

        // Regex settings
        regex_pattern: Option<String>, // Current regex pattern
        pub matched_rows: Vec<usize>,  // List of index of matches when regex filtering is active
        pub last_index_regexed: usize,     // The last index the filtering function saw

        // Parser settings
        // parser: ???  // Reference to the current parser
        parser_index: usize,                // Index for the parser to look at
        parsed_messages: Vec<String>,       // List of parsed messages
        analytics_enabled: bool,            // Whetehr we are calcualting stats or not
        last_index_processed: usize,        // The last index the parsing function saw
        insert_mode: bool,                  // Default to insert mode (like vim) off
        current_status: String,             // Current status, aka what is in the command line
        highlight_match: bool,              // Determines whether we highlight the match to the user
        pub stick_to_bottom: bool,          // Whether we should follow the stream
        pub stick_to_top: bool, // Whether we should stick to the top and not render new lines
        pub manually_controlled_line: bool, // Whether manual scroll is active
        pub current_end: usize, // Current last row we have rendered
        streams: Vec<InputStream>, // Can be a vector of FileInputs, CommandInputs, etc
    }

    pub struct MainWindow {
        pub config: LogiraConfig,
        pub input_type: InputType,
        stdscr: Option<ncurses::WINDOW>,
        output: Option<ncurses::WINDOW>,
        input: Option<ncurses::WINDOW>,
        length_finder: LengthFinder,
    }

    impl MainWindow {
        fn build_streams(&self, commands: Vec<String>) -> Vec<InputStream> {
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

        pub fn new(cache: bool, smart_poll_rate: bool) -> MainWindow {
            // Build streams here
            MainWindow {
                stdscr: None,
                input_type: InputType::Normal,
                output: None,
                input: None,
                length_finder: LengthFinder::new(),
                config: LogiraConfig {
                    poll_rate: FASTEST,
                    smart_poll_rate: smart_poll_rate,
                    first_run: true,
                    height: 0,
                    width: 0,
                    loop_time: 0.0,
                    previous_render: (0, 0),
                    previous_messages: None,
                    exit_val: 0,
                    stderr_messages: vec![], // fix
                    stdout_messages: vec![], // fix
                    stream_type: StreamType::StdErr,
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
                    streams: vec![],
                },
            }
        }

        fn determine_render_position(&mut self) -> (usize, usize) {
            let mut end: usize = 0;
            let mut rows: usize = 0;
            let mut message_pointer_length: usize = self.messages().len();

            if self.config.stick_to_top {
                let mut current_index: usize = 0;
                loop {
                    let message: &str = match self.input_type {
                        InputType::Normal | InputType::MultipleChoice | InputType::Command => {
                            &self.messages()[current_index]
                        }
                        InputType::Parser | InputType::Regex => {
                            message_pointer_length = self.config.matched_rows.len();
                            &self.messages()[self.config.matched_rows[current_index]]
                        }
                    };

                    // Determine if we can fit the next message
                    // TODO: Fix Cast here
                    let message_length = self.length_finder.get_real_length(message);
                    rows += max(
                        1,
                        (message_length + (self.config.width as usize - 1))
                            / self.config.width as usize,
                    );

                    // TODO: broken insertion for blank lines!
                    if message_length == 0 {
                        rows = match rows.checked_add(1) {
                            Some(value) => value,
                            None => break,
                        };
                    }

                    // If we can fit, increment the last row number
                    if rows <= self.config.last_row as usize
                        && current_index < message_pointer_length - 1
                    {
                        current_index += 1;
                        continue;
                    }

                    // If the above if doesn't hit, we are done
                    break;
                }
                self.config.current_end = current_index; // Save this row so we know where we are
                return (0, current_index);
            } else if self.config.stick_to_bottom {
                end = message_pointer_length;
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
            let start = max(0, end as i32 - self.config.last_row - 1) as usize;
            (start, end)
        }

        fn render_text_in_output(&mut self) {
            let mut current_row = self.config.last_row as usize;
            let width = self.config.width as usize;

            // Determine the start and end position of the render
            let (start, end) = self.determine_render_position();

            // Don't do anything if nothing changed; start at index 0
            if !self.config.analytics_enabled && self.config.previous_render == (max(0, start), end)
            {
                return;
            }

            // Lock in the previous render state
            self.config.previous_render = (max(0, start), end);
            self.reset_output();

            // Implement the rest of the rendering algorithm
            // Main issue is determining which vec we are reading the data from and adjusting as a result
            // panic!("{:?}, {:?}", start, end);
            for index in (start..end).rev() {
                let message: &str = match self.input_type {
                    InputType::Normal | InputType::MultipleChoice | InputType::Command => {
                        &self.messages()[index]
                    }
                    InputType::Parser | InputType::Regex => {
                        &self.messages()[self.config.matched_rows[index]]
                    }
                };

                let message_length = self.length_finder.get_real_length(message);
                current_row =
                    match current_row.checked_sub(max(1, (message_length + (width - 1)) / width)) {
                        Some(value) => value,
                        None => break,
                    };

                // TODO: broken insertion for blank lines!
                if message.len() == 0 {
                    current_row = match current_row.checked_sub(1) {
                        Some(value) => value,
                        None => break,
                    };
                }

                // TODO: handle color codes
                mvwaddstr(self.screen(), current_row as i32, 0, message);
            }
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

        pub fn messages(&self) -> &Vec<String> {
            match self.config.stream_type {
                StreamType::StdErr => &self.config.stderr_messages,
                StreamType::StdOut => &self.config.stdout_messages,
            }
        }

        pub fn go_to_cli(&self) {
            mv(self.config.height - 2, 1);
        }

        /// Overwrites the output window with empty space
        /// TODO: faster?
        fn reset_output(&self) {
            let clear = " ".repeat((self.config.width) as usize); // TODO: Store this string as a class attribute, recalc on resize

            for row in 0..self.config.last_row {
                mvwaddstr(self.screen(), row as i32, 0, &clear);
            }
        }

        pub fn reset_command_line(&self) {
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

        /// Set the first col of the command line depending on mode
        pub fn set_cli_cursor(&self, content: Option<u32>) {
            self.go_to_cli();
            let first_char = match self.input_type {
                InputType::Normal => ncurses::ACS_VLINE(),
                InputType::MultipleChoice => content.unwrap_or(cli_chars::mc_char),
                InputType::Command => content.unwrap_or(cli_chars::command_char),
                InputType::Regex => content.unwrap_or(cli_chars::regex_char),
                InputType::Parser => content.unwrap_or(cli_chars::parser_char),
            };
            mvwaddch(self.screen(), self.config.last_row + 1, 0, first_char);
        }

        /// Set dimensions
        fn update_dimensions(&mut self) {
            getmaxyx(
                self.screen(),
                &mut self.config.height,
                &mut self.config.width,
            );
            self.config.last_row = self.config.height - 3;
        }

        pub fn start(&mut self, commands: Vec<String>) {
            // Build the app
            self.config.streams = self.build_streams(commands);

            // Build the UI, get reference to the text body content, etc
            self.stdscr = Some(init_scr());
            ncurses::nodelay(self.screen(), true);

            // Hide the cursor
            curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

            // Set UI Size
            self.update_dimensions();

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

            // Initial message collection
            self.recieve_streams();

            // Default is StdErr, swap based on number of messages
            if self.config.stdout_messages.len() > self.config.stderr_messages.len() {
                self.config.stream_type = StreamType::StdOut;
            }

            // enum for input mode: {normal, command, regex, choice}
            // if input mode is command or regex, draw/remove the character to the command line
            // Otherwise, show status
            loop {
                // Update streams here
                let t_0 = Instant::now();
                let new_messages = self.recieve_streams();
                let t_1 = t_0.elapsed();
                // println!("{} in {:?}", new_messages, t_1);

                match ncurses::getch() {
                    -1 => (), // possibly sleep, cleanup, etc
                    input => match self.input_type {
                        InputType::Normal => normal_handler.recieve_input(self, input),
                        InputType::Command => command_handler.recieve_input(self, input),
                        InputType::Regex => regex_handler.recieve_input(self, input),
                        InputType::Parser => parser_handler.recieve_input(self, input),
                        InputType::MultipleChoice => mc_handler.recieve_input(self, input),
                    },
                }
                self.render_text_in_output();
                use std::{thread, time};
                let sleep = time::Duration::from_millis(100);
                thread::sleep(sleep);
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::MainWindow;
        fn dummy_logria() -> MainWindow {
            let mut app = MainWindow::new(true, true);

            // Set dimensions
            app.config.height = 10;
            app.config.width = 100;

            // Set fake previous render
            app.config.last_row = app.config.height - 3; // simulate the last row we can render to
            app.config.current_end = 80; // Simulate the last message rendered

            // Set fake messages
            app.config.stderr_messages = (0..100).map(|x| x.to_string()).collect();

            app
        }

        #[test]
        fn test_render_final_items() {
            let mut logria = dummy_logria();

            // Set scroll state
            logria.config.manually_controlled_line = false;
            logria.config.stick_to_top = false;
            logria.config.stick_to_bottom = true;

            let (start, end) = logria.determine_render_position();
            assert_eq!(start, 92);
            assert_eq!(end, 100);
        }

        #[test]
        fn test_render_first_items() {
            let mut logria = dummy_logria();

            // Set scroll state
            logria.config.manually_controlled_line = false;
            logria.config.stick_to_top = true;
            logria.config.stick_to_bottom = false;

            let (start, end) = logria.determine_render_position();
            assert_eq!(start, 0);
            assert_eq!(end, 7);
        }

        #[test]
        fn test_render_few_from_middle() {
            let mut logria = dummy_logria();

            // Set scroll state
            logria.config.manually_controlled_line = true;
            logria.config.stick_to_top = false;
            logria.config.stick_to_bottom = false;

            // Set current scroll state
            logria.config.current_end = 3;

            // Set small content
            logria.config.stderr_messages = (0..4).map(|x| x.to_string()).collect();

            let (start, end) = logria.determine_render_position();
            assert_eq!(start, 0);
            assert_eq!(end, 3);
        }

        #[test]
        fn test_render_from_middle() {
            let mut logria = dummy_logria();

            // Set scroll state
            logria.config.manually_controlled_line = true;
            logria.config.stick_to_top = false;
            logria.config.stick_to_bottom = false;

            // Set current scroll state
            logria.config.current_end = 80;
            let (start, end) = logria.determine_render_position();
            assert_eq!(start, 72);
            assert_eq!(end, 80);
        }

        #[test]
        fn test_render_from_middle_early() {
            let mut logria = dummy_logria();

            // Set scroll state
            logria.config.manually_controlled_line = true;
            logria.config.stick_to_top = false;
            logria.config.stick_to_bottom = false;

            // Set current scroll state
            logria.config.current_end = 5;

            let (start, end) = logria.determine_render_position();
            assert_eq!(start, 0);
            assert_eq!(end, 5);
        }

        #[test]
        fn test_render_small_from_top() {
            let mut logria = dummy_logria();

            // Set scroll state
            logria.config.manually_controlled_line = false;
            logria.config.stick_to_top = true;
            logria.config.stick_to_bottom = false;

            // Set current scroll state
            logria.config.current_end = 0;

            // Set small content
            logria.config.stderr_messages = (0..6).map(|x| x.to_string()).collect();

            let (start, end) = logria.determine_render_position();
            assert_eq!(start, 0);
            assert_eq!(end, 5);
        }
    }
}
