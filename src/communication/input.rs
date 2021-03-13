pub mod stream {
    use std::{
        error::Error,
        fs::File,
        io::{BufRead, BufReader},
        path::Path,
        process::Stdio,
        sync::{
            mpsc::{channel, Receiver},
            Arc, Mutex,
        },
        thread, time,
    };

    use tokio::{
        io::{AsyncBufReadExt, BufReader as TokioBufReader},
        process::Command,
        runtime::Runtime,
    };

    use crate::constants::{cli::poll_rate::FASTEST, directories::home};

    #[derive(Debug)]
    pub struct InputStream {
        pub stdout: Receiver<String>,
        pub stderr: Receiver<String>,
        pub proccess_name: String,
        pub process: Result<std::thread::JoinHandle<()>, std::io::Error>,
        _type: String,
    }

    pub trait Input {
        fn new(poll_rate: Option<u64>, name: String, command: String) -> InputStream;
    }

    #[derive(Debug)]
    pub struct FileInput {}

    impl Input for FileInput {
        fn new(_: Option<u64>, name: String, command: String) -> InputStream {
            // Setup multiprocessing queues
            let (_, err_rx) = channel();
            let (out_tx, out_rx) = channel();

            // Start process
            let process = thread::Builder::new()
                .name(String::from(format!("FileInput: {}", name)))
                .spawn(move || {
                    // Remove, as file input should be immediately buffered...
                    let path = Path::new(&command);

                    // Try and open a handle to the file
                    let file = match File::open(&path) {
                        // The `description` method of `io::Error` returns a string that describes the error
                        Err(why) => panic!("Couldn't open {:?}: {}", path, Error::to_string(&why)),
                        Ok(file) => file,
                    };

                    // Create a buffer and read from it
                    let reader = BufReader::new(file);
                    for line in reader.lines() {
                        if line.is_ok() {
                            out_tx.send(line.unwrap()).unwrap();
                        }
                    }
                });

            InputStream {
                stdout: out_rx,
                stderr: err_rx,
                proccess_name: name,
                process: process,
                _type: String::from("FileInput"),
            }
        }
    }

    #[derive(Debug)]
    pub struct CommandInput {}

    impl CommandInput {
        /// Parse a command string to a list of parts for `subprocess`
        fn parse_command(command: &str) -> Vec<&str> {
            command.split(" ").collect()
        }
    }

    impl Input for CommandInput {
        fn new(poll_rate: Option<u64>, name: String, command: String) -> InputStream {
            // Setup multiprocessing queues
            let (err_tx, err_rx) = channel();
            let (out_tx, out_rx) = channel();

            // Handle poll rate
            let poll_rate = Arc::new(Mutex::new(poll_rate.unwrap_or(FASTEST)));
            let internal_poll_rate = Arc::clone(&poll_rate);

            // Start reading from the queues
            let process = thread::Builder::new()
                .name(String::from(format!("CommandInput: {}", name)))
                .spawn(move || {
                    let runtime = Runtime::new().unwrap();
                    runtime.block_on(async {
                        let command_to_run = CommandInput::parse_command(&command);
                        let mut proc_read = match Command::new(command_to_run[0])
                            .args(&command_to_run[1..])
                            .current_dir(home())
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .spawn()
                        {
                            Ok(connected) => connected,
                            Err(why) => panic!("Unable to connect to process: {}", why),
                        };

                        // Create buffers from stderr and stdout handles
                        let mut stdout =
                            TokioBufReader::new(proc_read.stdout.take().unwrap()).lines();
                        let mut stderr =
                            TokioBufReader::new(proc_read.stderr.take().unwrap()).lines();

                        loop {
                            let wait = internal_poll_rate.lock().unwrap();
                            thread::sleep(time::Duration::from_millis(*wait));

                            tokio::select! {
                                Ok(line) = stdout.next_line() => {
                                    match line {
                                        Some(l) =>  out_tx.send(l).unwrap(),
                                        None => {},
                                    }
                                }
                                Ok(line) = stderr.next_line() => {
                                    match line {
                                        Some(l) =>  err_tx.send(l).unwrap(),
                                        None => {},
                                    }
                                }
                            }
                        }
                    });
                });

            InputStream {
                stdout: out_rx,
                stderr: err_rx,
                proccess_name: name,
                process: process,
                _type: String::from("CommandInput"),
            }
        }
    }

    pub fn build_streams(commands: Vec<String>) -> Vec<InputStream> {
        let mut streams: Vec<InputStream> = vec![];
        for command in commands {
            // Determine if command is a file, create FileInput if it is, CommandInput if not
            let path = Path::new(&command);
            match path.exists() {
                true => {
                    // Additional convetsion because file_name() generates OSString
                    let name = path.file_name().unwrap().to_str().unwrap().to_string();
                    // None indicates default poll rate
                    streams.push(FileInput::new(None, name, command));
                }
                false => {
                    let name = path.to_str().unwrap().to_string();
                    // None indicates default poll rate
                    streams.push(CommandInput::new(None, name, command));
                }
            }
        }
        streams
    }

    #[cfg(test)]
    mod tests {
        use super::build_streams;

        #[test]
        fn test_build_file_stream() {
            let commands = vec![String::from("README.md")];
            let streams = build_streams(commands);
            assert_eq!(streams[0]._type, "FileInput");
        }

        #[test]
        fn test_build_command_stream() {
            let commands = vec![String::from("ls -la ~")];
            let streams = build_streams(commands);
            assert_eq!(streams[0]._type, "CommandInput");
        }

        #[test]
        fn test_build_command_and_file_streams() {
            let commands = vec![String::from("ls -la ~"), String::from("README.md")];
            let streams = build_streams(commands);
            assert_eq!(streams[0]._type, "CommandInput");
            assert_eq!(streams[1]._type, "FileInput");
        }

        #[test]
        fn test_build_multiple_command_streams() {
            let commands = vec![String::from("ls -la ~"), String::from("ls /")];
            let streams = build_streams(commands);
            assert_eq!(streams[0]._type, "CommandInput");
            assert_eq!(streams[1]._type, "CommandInput");
        }

        #[test]
        fn test_build_multiple_file_streams() {
            let commands = vec![String::from("README.md"), String::from("Cargo.toml")];
            let streams = build_streams(commands);
            assert_eq!(streams[0]._type, "FileInput");
            assert_eq!(streams[1]._type, "FileInput");
        }
    }
}

pub mod input_type {
    pub enum InputType {
        Normal,
        Command,
        Regex,
        Parser,
        Startup,
        MultipleChoice,
    }
}

pub mod stream_type {
    #[derive(Debug)]
    pub enum StreamType {
        StdErr,
        StdOut,
        Startup
    }
}
