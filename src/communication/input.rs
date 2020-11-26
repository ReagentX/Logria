pub mod stream {
    use std::error::Error;
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::path::Path;
    use std::sync::mpsc::{channel, Receiver};
    use std::sync::{Arc, Mutex};
    use std::thread;

    use subprocess::{Popen, PopenConfig, Redirection};

    use crate::constants::cli::poll_rate::FASTEST;

    #[derive(Debug)]
    pub struct InputStream {
        pub poll_rate: Arc<Mutex<u64>>,
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
        fn new(poll_rate: Option<u64>, name: String, command: String) -> InputStream {
            // Setup multiprocessing queues
            let (_, err_rx) = channel();
            let (out_tx, out_rx) = channel();

            // Start process
            let poll_rate = Arc::new(Mutex::new(poll_rate.unwrap_or(FASTEST)));
            let internal_poll_rate = Arc::clone(&poll_rate);
            let process = thread::Builder::new()
                .name(name.to_string())
                .spawn(move || {
                    // Remove, as file input should be immediately buffered...
                    let num = internal_poll_rate.lock().unwrap();
                    let path = Path::new(&command);

                    // Try and open a handle to the file
                    let file = match File::open(&path) {
                        // The `description` method of `io::Error` returns a string that describes the error
                        Err(why) => panic!("couldn't open {:?}: {}", path, Error::to_string(&why)),
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
                poll_rate: poll_rate,
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
        fn resolve_command(command: &str) -> Vec<String> {
            vec![]
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
                .name(name.to_string())
                .spawn(move || {
                    let mut proc_read = match Popen::create(
                        &CommandInput::resolve_command(&command),
                        PopenConfig {
                            stdout: Redirection::Pipe,
                            stderr: Redirection::Pipe,
                            ..Default::default()
                        },
                    ) {
                        Ok(connected) => connected,
                        Err(why) => panic!("Unable to connect to process: {}", why),
                    };

                    loop {
                        let output = proc_read.communicate(None);
                        match output {
                            Ok(output) => {
                                let (stdout_content, stderr_content) = output;
                                if stderr_content.is_some() {
                                    err_tx.send(stderr_content.unwrap()).unwrap();
                                }
                                if stdout_content.is_some() {
                                    out_tx.send(stdout_content.unwrap()).unwrap();
                                }
                            }
                            Err(_) => {
                                // No more data to read, end the pipe
                                break;
                            }
                        }
                    }
                }); // TODO

            InputStream {
                poll_rate: poll_rate,
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
            let commands = vec![String::from("ls -lq")];
            let streams = build_streams(commands);
            assert_eq!(streams[0]._type, "CommandInput");
        }
    }
}

pub mod input_type {
    pub enum InputType {
        Normal,
        Command,
        Regex,
        Parser,
        MultipleChoice,
    }
}

pub mod stream_type {
    #[derive(Debug)]
    pub enum StreamType {
        StdErr,
        StdOut,
    }
}
