use std::{
    cmp::max,
    io::{stdout, Write},
    panic,
    time::{Duration, Instant},
};

use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    execute, queue, style,
    terminal::{disable_raw_mode, size, Clear, ClearType},
    Result,
};
use regex::bytes::Regex;

use crate::{
    communication::{
        handlers::{
            command::CommandHandler,
            handler::Handler,
            multiple_choice::MultipleChoiceHandler,
            normal::NormalHandler,
            parser::{ParserHandler, ParserState},
            processor::ProcessorMethods,
            regex::RegexHandler,
            startup::StartupHandler,
        },
        input::{build_streams_from_input, InputStream, InputType, StreamType},
    },
    constants::cli::{
        cli_chars, colors,
        messages::{NO_MESSAGE_IN_BUFFER_NORMAL, NO_MESSAGE_IN_BUFFER_PARSER, PIPE_INPUT_ERROR},
        poll_rate::DEFAULT,
    },
    ui::{
        interface::{build, valid_tty},
        scroll::ScrollState,
    },
    util::{
        poll::{ms_per_message, RollingMean},
        sanitizers::length::LengthFinder,
        types::Del,
    },
};

pub struct LogriaConfig {
    /// Window width
    pub width: u16,
    /// Window height
    pub height: u16,
    /// The last row we can render, aka number of lines visible in the tty
    pub last_row: u16,
    /// Current last row we have rendered
    pub current_end: usize,

    // Message buffers
    /// Messages read from standard error
    stderr_messages: Vec<String>,
    /// Messages read from standard output
    stdout_messages: Vec<String>,
    /// The stream type Logria is currently displaying
    pub stream_type: StreamType,
    /// The previous stream the user was looking at
    pub previous_stream_type: StreamType,
    /// Messages displayed by extensions
    pub auxiliary_messages: Vec<String>,

    // Regex settings
    /// Current regex pattern
    pub regex_pattern: Option<Regex>,
    /// List of index of matches when regex filtering is active
    pub matched_rows: Vec<usize>,
    /// The last index the filtering function saw
    pub last_index_regexed: usize,
    /// A regex to remove ANSI color codes
    color_replace_regex: Regex,
    /// Determines whether we highlight the matched text to the user
    pub highlight_match: bool,

    // Parser settings
    /// Index for the parser to look at
    pub parser_index: usize,
    /// The state of the current parser
    pub parser_state: ParserState,
    /// Whether we are aggregating log data or not
    pub aggregation_enabled: bool,
    /// The last index the parsing function saw
    pub last_index_processed: usize,
    /// The number of items to get when aggregating a Counter
    pub num_to_aggregate: usize,

    // App state
    /// How long a loop of the main app takes
    loop_time: Instant,
    /// The rate at which we check for new messages
    pub poll_rate: u64,
    /// A deque based moving average tracker
    pub message_speed_tracker: RollingMean,
    /// Whether we reduce the poll rate to the message receive speed
    pub smart_poll_rate: bool,
    /// Whether the app records user input to a history tape
    pub use_history: bool,

    // Render data
    /// The current scroll mode
    pub scroll_state: ScrollState,
    /// Can be a vector of FileInputs, CommandInputs, etc
    pub streams: Vec<InputStream>,
    /// Tuple of previous render boundaries, i.e. the (start, end) range of buffer that is rendered
    previous_render: (usize, usize),
    /// True if the previously rendered buffer had no data in it, False otherwise
    was_empty: bool,
    /// True if we just swapped input types, False otherwise
    pub did_switch: bool,
    /// Pointer to function used to delete items for the `: r` command
    pub delete_func: Del,
    /// Current status of the app  if there is one, i.e. if regex or parsers are active
    pub current_status: Option<String>,
    /// Function that can generate messages for display
    pub generate_auxiliary_messages: Option<fn() -> Vec<String>>,
}

pub struct MainWindow {
    pub config: LogriaConfig,
    pub input_type: InputType,
    pub previous_input_type: InputType,
    // pub output: Stdout,
    pub mc_handler: MultipleChoiceHandler,
    length_finder: LengthFinder,
}

impl MainWindow {
    /// Construct sample window for testing simple actions
    pub fn _new_dummy() -> MainWindow {
        let mut app = MainWindow::new(true, true);

        // Set fake dimensions
        app.config.height = 10;
        app.config.width = 100;
        app.config.stream_type = StreamType::StdErr;
        app.config.previous_stream_type = StreamType::StdOut;

        // Set fake previous render
        app.config.last_row = app.config.height - 3; // simulate the last row we can render to

        // Set fake messages
        app.config.stderr_messages = (0..100).map(|x| x.to_string()).collect();

        app
    }

    /// Construct sample window for testing parsers
    pub fn _new_dummy_parse() -> MainWindow {
        let mut app = MainWindow::new(true, true);

        // Set fake dimensions
        app.config.height = 10;
        app.config.width = 100;
        app.config.stream_type = StreamType::StdErr;
        app.config.previous_stream_type = StreamType::StdOut;

        // Set fake previous render
        app.config.last_row = app.config.height - 3; // simulate the last row we can render to

        // Set fake messages
        app.config.stderr_messages = (10..110)
            .map(|x| format!("{} - {} - {} - {}", x, x - 1, x - 2, x - 3))
            .collect();

        app
    }

    /// Construct sample window for testing date parsers
    pub fn _new_dummy_parse_date() -> MainWindow {
        let mut app = MainWindow::new(true, true);

        // Set fake dimensions
        app.config.height = 10;
        app.config.width = 100;
        app.config.stream_type = StreamType::StdErr;
        app.config.previous_stream_type = StreamType::StdOut;

        // Set fake previous render
        app.config.last_row = app.config.height - 3; // simulate the last row we can render to

        // Set fake messages
        app.config.stderr_messages = vec![
            "2021-03-10 | 08:10:26 | 2021-03-19 08:10:26".to_string(),
            "2021-03-12 | 08:10:36 | 2021-03-19 08:12:26".to_string(),
            "2021-03-12 | 08:10:46 | 2021-03-19 09:14:26".to_string(),
            "2021-03-15 | 08:10:56 | 2021-03-19 10:30:26".to_string(),
        ];

        app
    }

    pub fn new(history: bool, smart_poll_rate: bool) -> MainWindow {
        // Build streams here
        MainWindow {
            input_type: InputType::Startup,
            previous_input_type: InputType::Startup,
            length_finder: LengthFinder::new(),
            mc_handler: MultipleChoiceHandler::new(),
            config: LogriaConfig {
                poll_rate: DEFAULT,
                smart_poll_rate,
                use_history: history,
                height: 0,
                width: 0,
                loop_time: Instant::now(),
                previous_render: (0, 0),
                stderr_messages: vec![],
                stdout_messages: vec![],
                auxiliary_messages: vec![],
                stream_type: StreamType::Auxiliary,
                previous_stream_type: StreamType::Auxiliary,
                regex_pattern: None,
                matched_rows: vec![],
                last_index_regexed: 0,
                color_replace_regex: Regex::new(
                    crate::constants::cli::patterns::ANSI_COLOR_PATTERN,
                )
                .unwrap(),
                parser_index: 0,
                parser_state: ParserState::Disabled,
                aggregation_enabled: false,
                num_to_aggregate: 5,
                last_index_processed: 0,
                highlight_match: false,
                last_row: 0,
                scroll_state: ScrollState::Bottom,
                current_end: 0,
                streams: vec![],
                did_switch: false,
                was_empty: false,
                delete_func: None,
                generate_auxiliary_messages: None,
                current_status: None,
                message_speed_tracker: RollingMean::new(5),
            },
        }
    }

    /// Get the number of messages in the current message buffer
    pub fn number_of_messages(&self) -> usize {
        // if there is a regex active, use that, otherwise handle normally
        if self.config.regex_pattern.is_some() {
            return self.config.matched_rows.len();
        }
        match self.input_type {
            InputType::Normal | InputType::Command | InputType::Startup => self.messages().len(),
            InputType::Regex => {
                if self.config.regex_pattern.is_none() {
                    self.messages().len()
                } else {
                    self.config.matched_rows.len()
                }
            }
            InputType::Parser => {
                if self.config.parser_state == ParserState::Full {
                    self.config.auxiliary_messages.len()
                } else {
                    self.messages().len()
                }
            }
        }
    }

    /// Determine the start and end indexes we need to render in the window
    pub fn determine_render_position(&mut self) -> (usize, usize) {
        let mut end: usize = 0;
        let mut rows: usize = 0;
        let message_pointer_length = self.number_of_messages();

        // Handle empty message queue
        if message_pointer_length == 0 {
            return (0, 0);
        }

        // Early escape: render all if we have fewer messages than rows
        if message_pointer_length <= self.config.last_row as usize {
            return (0, message_pointer_length);
        }

        // Otherwise, determine how much we can render
        match self.config.scroll_state {
            ScrollState::Top => {
                let mut current_index: usize = 0;
                loop {
                    let message: &str = match self.input_type {
                        InputType::Normal | InputType::Command | InputType::Startup => {
                            &self.messages()[current_index]
                        }
                        InputType::Regex => {
                            // If we have not activated regex or parser yet, render normal messages
                            if self.config.regex_pattern.is_none() {
                                &self.messages()[current_index]
                            } else {
                                &self.messages()[self.config.matched_rows[current_index]]
                            }
                        }
                        InputType::Parser => &self.config.auxiliary_messages[current_index],
                    };

                    // Determine if we can fit the next message
                    let message_length = self.length_finder.get_real_length(message);
                    rows += max(
                        1,
                        (message_length + (self.config.width as usize - 2))
                            / self.config.width as usize,
                    );

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
            }
            ScrollState::Free => {
                if message_pointer_length < self.config.last_row as usize {
                    // If have fewer messages than lines, just render it all
                    end = message_pointer_length - 1;
                } else if (self.config.current_end < self.config.last_row as usize)
                    | (self.config.current_end < message_pointer_length)
                {
                    // If the last row we rendered comes before the last row we can render,
                    // use all of the available rows
                    end = self.config.current_end;
                } else {
                    // If we have overscrolled, go back
                    if self.config.current_end > message_pointer_length {
                        self.config.current_end = message_pointer_length;
                    } else {
                        // Since current_end can be zero, we have to use the number of messages
                        end = message_pointer_length;
                    }
                }
            }
            ScrollState::Bottom => {
                end = message_pointer_length;
            }
        }
        self.config.current_end = end; // Save this row so we know where we are
        let mut start: usize = 0; // default start
        if end > self.config.last_row as usize {
            start = (end.checked_sub(self.config.last_row as usize)).unwrap_or(start);
        }
        (start, end)
    }

    /// Get the message at a specific index in the current buffer
    fn get_message_at_index(&self, index: usize) -> &str {
        // if there is a regex active, use that, otherwise handle normally
        if self.config.regex_pattern.is_some() {
            return &self.messages()[self.config.matched_rows[index]];
        }
        match self.input_type {
            InputType::Normal | InputType::Command | InputType::Startup => &self.messages()[index],
            InputType::Regex => {
                if self.config.regex_pattern.is_none() {
                    &self.messages()[index]
                } else {
                    &self.messages()[self.config.matched_rows[index]]
                }
            }
            InputType::Parser => &self.messages()[index],
        }
    }

    /// Highlight the regex matched text with an ASCII escape code
    fn highlight_match(&self, message: &str) -> String {
        // Regex out any existing color codes
        // We use a bytes regex because we cannot compile the pattern using normal regex
        let clean_message = self
            .config
            .color_replace_regex
            .replace_all(message.as_bytes(), "".as_bytes());

        // Store some vectors of char bytes so we don't have to cast to a string every loop
        let mut new_msg: Vec<u8> = vec![];
        let mut last_end = 0;

        // Replace matched patterns with highlighted matched patterns
        for capture in self
            .config
            .regex_pattern
            .as_ref()
            .unwrap()
            .find_iter(&clean_message)
        {
            new_msg.extend(clean_message[last_end..capture.start()].to_vec());
            // Add start color string
            new_msg.extend(colors::HIGHLIGHT_COLOR.as_bytes().to_vec());
            new_msg.extend(clean_message[capture.start()..capture.end()].to_vec());
            // Add end color string
            new_msg.extend(colors::RESET_COLOR.as_bytes().to_vec());
            // Store the ending in case we have multiple matches so we can add the end later
            last_end = capture.end();
        }
        // Add on any extra chars and update the message String
        new_msg.extend(clean_message[last_end..].to_vec());
        String::from_utf8(new_msg).unwrap()
    }

    /// Render the relevant part of the message buffer in the window
    ///
    /// Adding padding and printing over the rest of the line is better than
    /// clearing the screen and writing again. This is because we can only fit
    /// a few items into the render queue. Because the queue is flushed
    /// automatically when it is full, we end up having a lot of partial screen
    /// renders, i.e. a lot of flickering, which makes for bad UX. This is not
    /// a perfect solution because we can still get partial renders if the
    /// terminal has a lot of lines, but we are guaranteed to never have blank
    /// lines in the render, which are what cause the flickering effect.
    fn render_text_in_output(&mut self) -> Result<()> {
        let mut stdout = stdout();
        // Save the cursor position (i.e. if the user is editing text in the command line)
        queue!(stdout, cursor::SavePosition)?;

        // Determine the start and end position of the render
        let (start, end) = self.determine_render_position();

        // If there are no messages in the buffer, tell the user
        // This will only ever hit once, because this method is only called if there are new
        // messages to render or a user action requires a full re-render
        if self.messages().is_empty() {
            match self.input_type {
                InputType::Parser => {
                    self.write_to_command_line(NO_MESSAGE_IN_BUFFER_PARSER)?;
                }
                InputType::Regex => {}
                _ => {
                    self.write_to_command_line(NO_MESSAGE_IN_BUFFER_NORMAL)?;
                }
            }
            self.config.was_empty = true;
            stdout.flush()?;
            return Ok(());
        }

        // Don't do anything if nothing changed; start at index 0
        if !self.config.aggregation_enabled && self.config.previous_render == (max(0, start), end) {
            queue!(stdout, cursor::RestorePosition)?;
            return Ok(());
        }

        // If the previous render was empty, we have a message in the command line
        // that we need to clear out, but only for modes where the status does not
        // come from user input
        if self.config.was_empty {
            self.config.was_empty = false;
            match self.input_type {
                InputType::Parser | InputType::Regex => {}
                _ => {
                    self.reset_command_line()?;
                }
            }
        }

        // Since we are rendering if we got here, lock in the new render state
        self.config.previous_render = (max(0, start), end);

        // Start the render from the last row
        let mut current_row = self.config.last_row;

        // Cast to usize so we can reference this instead of casting every time we need
        let width = self.config.width as usize;

        // Render each message from bottom to top
        for index in (start..end).rev() {
            // Get the next message from the message pointer
            let mut message = self.get_message_at_index(index);

            // Trim any spaces or newlines from the end of the message
            message = message.trim_end();

            // Get some metadata we need to render the message
            let message_length = self.length_finder.get_real_length(message);
            let message_rows = max(1, ((message_length) + (width - 1)) / width);

            // Update the current row, stop writing if there is no more space
            current_row = match current_row.checked_sub(max(1, message_rows as u16)) {
                Some(value) => value,
                None => break,
            };

            // See method docs for note on why we need this padding
            let message_padding_size = (width * message_rows) - message_length;
            let padding = " ".repeat(message_padding_size);

            if !(self.config.highlight_match && self.config.regex_pattern.is_some()) {
                // Render message normally
                queue!(
                    stdout,
                    cursor::MoveTo(0, current_row),
                    style::Print(message),
                    style::Print(padding)
                )?;
            } else {
                // Render message with highlight (additional allocation)
                queue!(
                    stdout,
                    cursor::MoveTo(0, current_row),
                    style::Print(self.highlight_match(message)),
                    style::Print(padding)
                )?;
            }
        }

        // Overwrite any new blank lines
        // We could iterate over (0..current_row), but we don't need to allocate clear_line
        if current_row > 0 {
            let clear_line = " ".repeat(width);
            (0..current_row).for_each(|row| {
                // No `?` here because it is inside of a closure
                queue!(stdout, cursor::MoveTo(0, row), style::Print(&clear_line),).unwrap()
            });
        }

        // Restore the cursor position and flush the queue
        queue!(stdout, cursor::RestorePosition)?;
        stdout.flush()?;
        Ok(())
    }

    /// Force render
    pub fn redraw(&mut self) -> Result<()> {
        self.config.previous_render = (0, 0);
        self.render_text_in_output()?;
        Ok(())
    }

    /// Get the previous message pointer
    pub fn previous_messages(&self) -> &Vec<String> {
        match self.config.previous_stream_type {
            StreamType::StdErr => &self.config.stderr_messages,
            StreamType::StdOut => &self.config.stdout_messages,
            StreamType::Auxiliary => &self.config.auxiliary_messages,
        }
    }

    /// Get the current message pointer
    pub fn messages(&self) -> &Vec<String> {
        match self.config.stream_type {
            StreamType::StdErr => &self.config.stderr_messages,
            StreamType::StdOut => &self.config.stdout_messages,
            StreamType::Auxiliary => &self.config.auxiliary_messages,
        }
    }

    /// Move the cursor to the CLI window
    pub fn go_to_cli(&mut self) -> Result<()> {
        let cli_position = self.config.height - 2;
        queue!(stdout(), cursor::MoveTo(1, cli_position))?;
        Ok(())
    }

    /// Set the output to command mode for command interpretation
    pub fn set_command_mode(&mut self, delete_func: Del) -> Result<()> {
        self.config.delete_func = delete_func;
        self.update_input_type(InputType::Command)?;
        self.go_to_cli()?;
        self.reset_command_line()?;
        self.set_cli_cursor(None)?;
        queue!(stdout(), cursor::Show)?;
        Ok(())
    }

    /// Writes the current status String to the command line if it exists
    pub fn write_status(&mut self) -> Result<()> {
        if self.config.current_status.is_some() {
            self.write_to_command_line(&self.config.current_status.as_ref().unwrap().to_owned())?;
        }
        Ok(())
    }

    /// Overwrites the output window with empty space
    /// TODO: faster?
    pub fn reset_output(&mut self) -> Result<()> {
        let last_row = self.config.last_row - 1;
        execute!(stdout(), cursor::SavePosition)?;
        queue!(
            stdout(),
            cursor::MoveTo(1, last_row),
            Clear(ClearType::CurrentLine),
            Clear(ClearType::FromCursorUp),
        )?;
        execute!(stdout(), cursor::RestorePosition)?;
        Ok(())
    }

    /// Empty the command line
    pub fn reset_command_line(&mut self) -> Result<()> {
        // Leave padding for surrounding rectangle, we cannot use deleteln because it destroys the rectangle
        // TODO: Store this string as a class attribute, re-calculate on resize
        let clear = " ".repeat((self.config.width - 2) as usize);
        self.go_to_cli()?;

        // If the cursor was visible, hide it
        queue!(stdout(), style::Print(&clear), cursor::Hide)?;
        Ok(())
    }

    /// Write text to the command line
    pub fn write_to_command_line(&mut self, content: &str) -> Result<()> {
        queue!(stdout(), cursor::SavePosition)?;
        // Remove what used to be in the command line
        self.reset_command_line()?;

        // Add the string to the front of the command line
        // TODO: Possibly validate length?
        self.go_to_cli()?;
        queue!(stdout(), style::Print(content), cursor::RestorePosition)?;
        Ok(())
    }

    /// Set the first col of the command line depending on mode
    pub fn set_cli_cursor(&mut self, content: Option<&'static str>) -> Result<()> {
        self.go_to_cli()?;
        let first_char = match self.input_type {
            InputType::Normal | InputType::Startup => content.unwrap_or(cli_chars::NORMAL_CHAR),
            InputType::Command => content.unwrap_or(cli_chars::COMMAND_CHAR),
            InputType::Regex => content.unwrap_or(cli_chars::REGEX_CHAR),
            InputType::Parser => content.unwrap_or(cli_chars::PARSER_CHAR),
        };

        // Write the CLI cursor in the command line bounding box
        let cli_char_vertical = self.config.last_row + 1;
        execute!(
            stdout(),
            cursor::MoveTo(0, cli_char_vertical),
            style::Print(first_char)
        )?;
        Ok(())
    }

    /// Redraw auxiliary text the given function pointer
    pub fn render_auxiliary_text(&mut self) -> Result<()> {
        if let Some(gen) = self.config.generate_auxiliary_messages {
            self.config.auxiliary_messages.clear();
            self.config.auxiliary_messages.extend(gen());
            self.redraw()?;
        } else {
            self.write_to_command_line("Cannot write auxiliary messages with no function!")?;
        }
        Ok(())
    }

    /// Set dimensions
    fn update_dimensions(&mut self) -> Result<()> {
        let (w, h) = size()?;
        self.config.height = h;
        self.config.width = w;
        self.config.last_row = self.config.height.checked_sub(3).unwrap_or(h);
        build(self)?;
        Ok(())
    }

    /// Set a new input type enum while preserving the old one in the history
    pub fn update_input_type(&mut self, input_type: InputType) -> Result<()> {
        self.previous_input_type = self.input_type;
        self.input_type = input_type;
        Ok(())
    }

    /// Determine a reasonable poll rate based on the speed of messages received
    fn handle_smart_poll_rate(&mut self, t_1: Duration, new_messages: u64) {
        if self.config.smart_poll_rate && !(self.input_type == InputType::Startup) {
            // Set the poll rate to the number of milliseconds per message
            self.config
                .message_speed_tracker
                .update(ms_per_message(t_1, new_messages));
            self.update_poll_rate(self.config.message_speed_tracker.mean());

            // Reset the timer we use to count new messages
            self.config.loop_time = Instant::now();
        }
    }

    /// Update poll rate of the main loop plus the child processes
    fn update_poll_rate(&mut self, new_poll_rate: u64) {
        self.config.poll_rate = new_poll_rate;
    }

    fn validate_environment(&self) {
        // Ensure the tty is valid before doing any work
        if !valid_tty() {
            /*
            Since we need to emit an error message, but the pipe to emit it through
            may be closed, we need to use a panic instead.
            This overrides the default panic handler to just emit a string, so it
            looks like a normal println
            */
            panic::set_hook(Box::new(|_| {
                println!("{}", PIPE_INPUT_ERROR);
            }));

            panic!();
        }
    }

    /// Initial application setup
    pub fn start(&mut self, commands: Option<Vec<String>>) -> Result<()> {
        self.validate_environment();

        // Build the app
        if let Some(c) = commands {
            // Build streams from the command used to launch Logria
            // If we cannot save to the disk, write to the command line and start without saving
            let possible_streams = build_streams_from_input(&c, true);
            match possible_streams {
                Ok(streams) => self.config.streams = streams,
                Err(why) => {
                    self.write_to_command_line(&why.to_string())?;
                    build_streams_from_input(&c, false).unwrap();
                }
            }

            // Set to display stderr by default
            self.config.previous_stream_type = StreamType::StdOut;
            self.config.stream_type = StreamType::StdErr;

            // Send input to normal handler
            self.input_type = InputType::Normal;
        }

        // Set UI Size
        self.update_dimensions()?;

        // Build the UI
        build(self)?;

        // Since building the UI hid the cursor, expose it again
        self.go_to_cli()?;
        execute!(stdout(), cursor::Show)?;

        // Start the main event loop
        self.main()?;
        Ok(())
    }

    /// Immediately exit the program
    pub fn quit(&mut self) -> Result<()> {
        execute!(stdout(), cursor::Show, Clear(ClearType::All))?;
        disable_raw_mode()?;
        for stream in &self.config.streams {
            *stream.should_die.lock().unwrap() = true;
        }
        std::process::exit(0);
    }

    /// Update stderr and stdout buffers from every stream's queue
    fn receive_streams(&mut self) -> u64 {
        let mut total_messages = 0;
        for stream in &self.config.streams {
            // Read from streams until there is no more input
            // ? May lock if logs come in too fast
            while let Ok(data) = stream.stderr.try_recv() {
                total_messages += 1;
                self.config.stderr_messages.push(data);
            }
            while let Ok(data) = stream.stdout.try_recv() {
                total_messages += 1;
                self.config.stdout_messages.push(data);
            }
        }
        total_messages
    }

    /// Main app loop
    fn main(&mut self) -> Result<()> {
        // Exit event
        let exit_key = KeyEvent {
            modifiers: KeyModifiers::CONTROL,
            code: KeyCode::Char('c'),
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        let refresh_key = KeyCode::F(5);

        // Instantiate handlers
        let mut normal_handler = NormalHandler::new();
        let mut command_handler = CommandHandler::new();
        let mut regex_handler = RegexHandler::new();
        let mut parser_handler = ParserHandler::new();
        let mut startup_handler = StartupHandler::new();

        // Setup startup messages
        self.config.generate_auxiliary_messages = Some(StartupHandler::get_startup_text);
        self.render_auxiliary_text()?;

        // Put the cursor in the command line
        self.go_to_cli()?;

        // Initial message collection
        self.receive_streams();

        // Default is StdErr, swap based on number of messages
        if self.config.stdout_messages.len() > self.config.stderr_messages.len() {
            self.config.stream_type = StreamType::StdOut;
        }

        // Render anything new in case the streams are already finished
        self.render_text_in_output()?;

        // Handle directing input to the correct handlers during operation
        loop {
            // Update streams and poll rate
            let num_new_messages = self.receive_streams();
            self.handle_smart_poll_rate(self.config.loop_time.elapsed(), num_new_messages);

            if poll(Duration::from_millis(self.config.poll_rate))? {
                match read()? {
                    Event::Key(input) => {
                        // Die on Ctrl-C
                        if input == exit_key {
                            self.quit()?;
                        }

                        // Otherwise, match input to action
                        match self.input_type {
                            InputType::Normal => normal_handler.receive_input(self, input.code)?,
                            InputType::Command => {
                                command_handler.receive_input(self, input.code)?
                            }
                            InputType::Regex => regex_handler.receive_input(self, input.code)?,
                            InputType::Parser => parser_handler.receive_input(self, input.code)?,
                            InputType::Startup => {
                                startup_handler.receive_input(self, input.code)?
                            }
                        }
                    }
                    Event::Mouse(_) => {} // Probably remove
                    Event::Resize(_, _) => {
                        self.update_dimensions()?;
                        self.redraw()?;
                    }
                    Event::FocusGained => {}
                    Event::FocusLost => {}
                    Event::Paste(_) => {}
                }
            }

            // Process matches if we just switched or if there are new messages
            if num_new_messages > 0 || self.config.did_switch {
                // Process extension methods
                match self.input_type {
                    InputType::Regex => {
                        if self.config.regex_pattern.is_some() {
                            regex_handler.process_matches(self)?;
                        } else if self.config.did_switch {
                            self.config.did_switch = false;
                        }
                    }
                    InputType::Parser => {
                        if self.config.parser_state == ParserState::Full {
                            parser_handler.process_matches(self)?;
                        }
                        if self.config.did_switch {
                            // 2 ticks, one to process the current input and another to refresh
                            // Did I just write a hack for my own app?
                            parser_handler.receive_input(self, refresh_key)?;
                            parser_handler.receive_input(self, refresh_key)?;
                            self.config.did_switch = false;
                        }
                    }
                    _ => {}
                }
                self.render_text_in_output()?;
            }
        }
    }
}

#[cfg(test)]
mod render_tests {
    use crate::{communication::reader::MainWindow, ui::scroll::ScrollState};

    #[test]
    fn test_render_final_items() {
        let mut logria = MainWindow::_new_dummy();

        // Set scroll state
        logria.config.scroll_state = ScrollState::Bottom;

        let (start, end) = logria.determine_render_position();
        assert_eq!(start, 93);
        assert_eq!(end, 100);
    }

    #[test]
    fn test_render_first_items() {
        let mut logria = MainWindow::_new_dummy();

        // Set scroll state
        logria.config.scroll_state = ScrollState::Top;

        let (start, end) = logria.determine_render_position();
        assert_eq!(start, 0);
        assert_eq!(end, 7);
    }

    #[test]
    fn test_render_few_from_middle() {
        let mut logria = MainWindow::_new_dummy();

        // Set scroll state
        logria.config.scroll_state = ScrollState::Free;

        // Set current scroll state
        logria.config.current_end = 3;

        // Set small content
        logria.config.stderr_messages = (0..4).map(|x| x.to_string()).collect();

        let (start, end) = logria.determine_render_position();
        assert_eq!(start, 0);
        assert_eq!(end, 4);
    }

    #[test]
    fn test_render_from_middle() {
        let mut logria = MainWindow::_new_dummy();

        // Set scroll state
        logria.config.scroll_state = ScrollState::Free;

        // Set current scroll state
        logria.config.current_end = 80;
        let (start, end) = logria.determine_render_position();
        assert_eq!(start, 73);
        assert_eq!(end, 80);
    }

    #[test]
    fn test_render_from_middle_early() {
        let mut logria = MainWindow::_new_dummy();

        // Set scroll state
        logria.config.scroll_state = ScrollState::Free;

        // Set current scroll state
        logria.config.current_end = 5;

        let (start, end) = logria.determine_render_position();
        assert_eq!(start, 0);
        assert_eq!(end, 5);
    }

    #[test]
    fn test_render_small_from_top() {
        let mut logria = MainWindow::_new_dummy();

        // Set scroll state
        logria.config.scroll_state = ScrollState::Top;

        // Set current scroll state
        logria.config.current_end = 0;

        // Set small content
        logria.config.stderr_messages = (0..6).map(|x| x.to_string()).collect();

        let (start, end) = logria.determine_render_position();
        assert_eq!(start, 0);
        assert_eq!(end, 6);
    }

    #[test]
    fn test_render_no_messages_top() {
        let mut logria = MainWindow::_new_dummy();

        // Set scroll state
        logria.config.scroll_state = ScrollState::Top;

        // Set current scroll state
        logria.config.current_end = 0;

        // Set small content
        logria.config.stderr_messages = vec![];

        let (start, end) = logria.determine_render_position();
        assert_eq!(start, 0);
        assert_eq!(end, 0);
    }

    #[test]
    fn test_render_no_messages_bottom() {
        let mut logria = MainWindow::_new_dummy();

        // Set scroll state
        logria.config.scroll_state = ScrollState::Bottom;

        // Set current scroll state
        logria.config.current_end = 0;

        // Set small content
        logria.config.stderr_messages = vec![];

        let (start, end) = logria.determine_render_position();
        assert_eq!(start, 0);
        assert_eq!(end, 0);
    }
}

#[cfg(test)]
mod poll_rate_tests {
    use crate::communication::{input::InputType, reader::MainWindow};
    use std::time::Duration;

    #[test]
    fn test_no_poll_rate_change_when_disabled() {
        let mut logria = MainWindow::_new_dummy();

        // Disable smart polling
        logria.config.smart_poll_rate = false;

        // Update the poll rate for 100ms
        logria.handle_smart_poll_rate(Duration::new(0, 100000000), 10);

        assert_eq!(logria.config.poll_rate, 50);
    }

    #[test]
    fn test_poll_rate_change_when_enabled_100ms_10messages() {
        let mut logria = MainWindow::_new_dummy();
        logria.input_type = InputType::Normal;

        // Test default value
        assert_eq!(logria.config.poll_rate, 50);

        // Update the poll rate
        logria.handle_smart_poll_rate(Duration::new(0, 100000000), 10);

        assert_eq!(logria.config.poll_rate, 10);
    }

    #[test]
    fn test_poll_rate_change_when_enabled_50ms_50messages() {
        let mut logria = MainWindow::_new_dummy();
        logria.input_type = InputType::Normal;

        // Test default value
        assert_eq!(logria.config.poll_rate, 50);

        // Update the poll rate
        logria.handle_smart_poll_rate(Duration::new(0, 50000000), 5);

        assert_eq!(logria.config.poll_rate, 10);
    }

    #[test]
    fn test_poll_rate_change_when_enabled_idle() {
        let mut logria = MainWindow::_new_dummy();
        logria.input_type = InputType::Normal;

        // Test default value
        assert_eq!(logria.config.poll_rate, 50);

        // Update the poll rate
        logria.handle_smart_poll_rate(Duration::new(0, 100000000), 0);
        logria.handle_smart_poll_rate(Duration::new(0, 100000000), 0);
        logria.handle_smart_poll_rate(Duration::new(0, 100000000), 0);
        logria.handle_smart_poll_rate(Duration::new(0, 100000000), 0);
        logria.handle_smart_poll_rate(Duration::new(0, 100000000), 0);
        logria.handle_smart_poll_rate(Duration::new(0, 100000000), 0);
        logria.handle_smart_poll_rate(Duration::new(0, 100000000), 0);

        assert_eq!(logria.config.poll_rate, 1000);
    }

    #[test]
    fn test_poll_rate_change_when_enabled_idle_multiple() {
        let mut logria = MainWindow::_new_dummy();
        logria.input_type = InputType::Normal;

        // Test default value
        assert_eq!(logria.config.poll_rate, 50);

        // Update the poll rate
        logria.handle_smart_poll_rate(Duration::new(0, 10000000), 1);

        assert_eq!(logria.config.poll_rate, 10);

        // Update the poll rate
        logria.handle_smart_poll_rate(Duration::new(0, 10000000), 1);

        assert_eq!(logria.config.poll_rate, 10);

        // Update the poll rate, don't go to 1000
        logria.handle_smart_poll_rate(Duration::new(0, 10000000), 0);

        assert_eq!(logria.config.poll_rate, 13);
    }
}
