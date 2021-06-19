pub mod stream {
    use std::{
        collections::HashSet,
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

    use crate::{
        constants::{cli::poll_rate::FASTEST, directories::home},
        extensions::session::{Session, SessionType},
    };

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
        /// Create a file input
        /// poll_rate is unused since the file will be read all at once
        fn new(_: Option<u64>, name: String, command: String) -> InputStream {
            // Setup multiprocessing queues
            let (_, err_rx) = channel();
            let (out_tx, out_rx) = channel();

            // Start process
            let process = thread::Builder::new()
                .name(format!("FileInput: {}", name))
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
                            out_tx
                                .send(match line {
                                    Ok(a) => a,
                                    _ => unreachable!(),
                                })
                                .unwrap();
                        }
                    }
                });

            InputStream {
                stdout: out_rx,
                stderr: err_rx,
                proccess_name: name,
                process,
                _type: String::from("FileInput"),
            }
        }
    }

    #[derive(Debug)]
    pub struct CommandInput {}

    impl CommandInput {
        /// Parse a command string to a list of parts for `subprocess`
        fn parse_command(command: &str) -> Vec<&str> {
            command.split(' ').collect()
        }
    }

    impl Input for CommandInput {
        /// Create a command input
        fn new(poll_rate: Option<u64>, name: String, command: String) -> InputStream {
            // Setup multiprocessing queues
            let (err_tx, err_rx) = channel();
            let (out_tx, out_rx) = channel();

            // Handle poll rate
            let poll_rate = Arc::new(Mutex::new(poll_rate.unwrap_or(FASTEST)));
            let internal_poll_rate = Arc::clone(&poll_rate);

            // Start reading from the queues
            let process = thread::Builder::new()
                .name(format!("CommandInput: {}", name))
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
                                    if let Some(l) = line { out_tx.send(l).unwrap() }
                                }
                                Ok(line) = stderr.next_line() => {
                                    if let Some(l) = line { err_tx.send(l).unwrap() }
                                }
                            }
                        }
                    });
                });

            InputStream {
                stdout: out_rx,
                stderr: err_rx,
                proccess_name: name,
                process,
                _type: String::from("CommandInput"),
            }
        }
    }

    fn determine_stream_type(command: &str) -> SessionType {
        // TODO: Fix logic, doesnt work for  "ls -lga"
        let path = Path::new(command);
        match path.exists() {
            true => SessionType::File,
            false => SessionType::Command,
        }
    }

    /// Build app streams from user input, i.e. command text or a filepath
    pub fn build_streams_from_input(commands: &Vec<String>, save: bool) -> Vec<InputStream> {
        let mut streams: Vec<InputStream> = vec![];
        let mut stream_types: HashSet<SessionType> = HashSet::new();
        for command in commands {
            // Determine if command is a file, create FileInput if it is, CommandInput if not
            match determine_stream_type(command) {
                SessionType::Command => {
                    // None indicates default poll rate
                    streams.push(CommandInput::new(
                        None,
                        command.to_owned(), // Same as the command
                        command.to_owned(),
                    ));
                    stream_types.insert(SessionType::File);
                }
                SessionType::File => {
                    // None indicates default poll rate
                    let path = Path::new(command);
                    let name = path.file_name().unwrap().to_str().unwrap().to_string();
                    streams.push(FileInput::new(None, name, command.to_owned()));
                    stream_types.insert(SessionType::File);
                }
                _ => {}
            }
        }
        if save {
            let stream_type = match stream_types.len() {
                1 => {
                    if stream_types.contains(&SessionType::File) {
                        SessionType::File
                    } else if stream_types.contains(&SessionType::Command) {
                        SessionType::Command
                    } else {
                        SessionType::Mixed
                    }
                }
                _ => SessionType::Mixed,
            };
            Session::new(commands, stream_type).save(&commands[0]);
        }
        streams
    }

    /// Build app streams from a session struct
    pub fn build_streams_from_session(session: Session) -> Vec<InputStream> {
        match session.stream_type {
            SessionType::Command => {
                let mut streams: Vec<InputStream> = vec![];
                session.commands.iter().for_each(|command| {
                    let name = command.to_string();
                    streams.push(CommandInput::new(None, name, command.to_owned()))
                });
                streams
            }
            SessionType::File => {
                let mut streams: Vec<InputStream> = vec![];
                session.commands.iter().for_each(|command| {
                    let name = command.to_string();
                    streams.push(FileInput::new(None, name, command.to_owned()))
                });
                streams
            }
            SessionType::Mixed => build_streams_from_input(&session.commands, false),
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::extensions::session::{Session, SessionType};

        use super::{build_streams_from_input, build_streams_from_session};

        #[test]
        fn test_build_file_stream() {
            let commands = vec![String::from("README.md")];
            let streams = build_streams_from_input(&commands, false);
            assert_eq!(streams[0]._type, "FileInput");
        }

        #[test]
        fn test_build_command_stream() {
            let commands = vec![String::from("ls -la ~")];
            let streams = build_streams_from_input(&commands, false);
            assert_eq!(streams[0]._type, "CommandInput");
        }

        #[test]
        fn test_build_command_and_file_streams() {
            let commands = vec![String::from("ls -la ~"), String::from("README.md")];
            let streams = build_streams_from_input(&commands, false);
            assert_eq!(streams[0]._type, "CommandInput");
            assert_eq!(streams[1]._type, "FileInput");
        }

        #[test]
        fn test_build_multiple_command_streams() {
            let commands = vec![String::from("ls -la ~"), String::from("ls /")];
            let streams = build_streams_from_input(&commands, false);
            assert_eq!(streams[0]._type, "CommandInput");
            assert_eq!(streams[1]._type, "CommandInput");
        }

        #[test]
        fn test_build_multiple_file_streams() {
            let commands = vec![String::from("README.md"), String::from("Cargo.toml")];
            let streams = build_streams_from_input(&commands, false);
            assert_eq!(streams[0]._type, "FileInput");
            assert_eq!(streams[1]._type, "FileInput");
        }

        #[test]
        fn test_build_file_stream_from_session() {
            let session = Session::new(&vec![String::from("README.md")], SessionType::File);
            let streams = build_streams_from_session(session);
            assert_eq!(streams[0]._type, "FileInput");
        }

        #[test]
        fn test_build_command_stream_from_session() {
            let session = Session::new(&vec![String::from("ls -l")], SessionType::Command);
            let streams = build_streams_from_session(session);
            assert_eq!(streams[0]._type, "CommandInput");
        }

        #[test]
        fn test_build_mixed_stream_from_session() {
            let session = Session::new(
                &vec![String::from("ls -l"), String::from("README.md")],
                SessionType::Mixed,
            );
            let streams = build_streams_from_session(session);
            assert_eq!(streams[0]._type, "CommandInput");
            assert_eq!(streams[1]._type, "FileInput");
        }
    }
}

pub mod input_type {
    #[derive(Debug)]
    pub enum InputType {
        Normal,
        Command,
        Regex,
        Parser,
        Auxiliary,
    }
}

pub mod stream_type {
    #[derive(Debug)]
    pub enum StreamType {
        StdErr,
        StdOut,
        Auxiliary,
    }
}
