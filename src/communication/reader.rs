pub mod main {
    use cursive::event::Event;
    use cursive::views::TextContent;
    use std::path::Path;

    use crate::communication::input::stream::{FileInput, InputStream};
    use crate::constants::cli::poll_rate::FASTEST;
    use crate::ui::windows::interface::build;

    pub struct LogiraConfig {
        pub poll_rate: u64,    // The rate at which we check for new messages
        smart_poll_rate: bool, // Whether we reduce the poll rate to the message receive speed
        first_run: bool,       // Whether this is a first run or not
        height: usize,         // Window height
        width: usize,          // Window width
        loop_time: f64,        // How long a loop of the main app takes
        messages: Option<&'static Vec<String>>, // Default to watching stderr
        previous_messages: Option<&'static Vec<String>>, // Pointer to the previous non-parsed message list, which is continuously updated
        exit_val: i8,                                    // If exit_val is -1, the app dies

        // Message buffers
        stderr_messages: Vec<String>,
        stdout_messages: Vec<String>,

        // Regex settings
        // func_handle:
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
        last_row: usize, // The last row we can render, aka number of lines visible in the tty
        stick_to_bottom: bool, // Whether we should follow the stream
        stick_to_top: bool, // Whether we should stick to the top and not render new lines
        manually_controlled_line: bool, // Whether manual scroll is active
        current_end: usize, // Current last row we have rendered
        stream: Vec<InputStream>, // Can be a vector of FileInputs, CommandInputs, etc
    }

    pub struct MainWindow {
        pub logria: cursive::Cursive,
        pub config: LogiraConfig,
        pub output: Option<TextContent>,
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
                logria: cursive::crossterm().unwrap(),
                output: None,
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

        pub fn start(&mut self) {
            // Build the UI, get reference to the text body content
            self.output = Some(build(self));

            // Set interal state
            let screen_resolution = self.logria.screen_size();
            self.config.height = screen_resolution.y;
            self.config.width = screen_resolution.x;
            self.config.last_row = self.config.height - 2;

            // Start the main event loop
            self.main();
        }

        fn main(&mut self) {
            // Main app loop
            loop {
                self.update_body("I got new content!!\nRust is fun!\nI think");
                // Logic goes here...
                // Need to figure out how to get/fire events between cursive and here.
                self.logria.on_event(Event::Char('q'));
                // I think events are not working because the text input field is always active

                self.logria.step();
            }
        }

        pub fn update_body(&mut self, content: &str) {
            match &self.output {
                Some(o) => o.set_content(content),
                None => panic!("There is no output window!"),
            };
        }
    }
}
