use std::{collections::HashSet, path::Path, result::Result};

use is_executable::is_executable;

use crate::{
    communication::input::streams::{CommandInput, FileInput, Input, InputStream},
    extensions::{
        extension::ExtensionMethods,
        session::{Session, SessionType},
    },
    util::error::LogriaError,
};

pub mod streams {
    use std::{
        env::current_dir,
        error::Error,
        fs::File,
        io::{BufRead, BufReader},
        path::Path,
        process::Stdio,
        result::Result,
        sync::mpsc::{channel, Receiver},
        thread, time,
    };

    use tokio::{
        io::{AsyncBufReadExt, BufReader as TokioBufReader},
        process::Command,
        runtime::Runtime,
    };

    use crate::util::{
        error::LogriaError,
        poll::{ms_per_message, RollingMean},
    };

    #[derive(Debug)]
    pub struct InputStream {
        pub stdout: Receiver<String>,
        pub stderr: Receiver<String>,
        pub process_name: String,
        pub process: Result<std::thread::JoinHandle<()>, std::io::Error>,
        pub _type: String,
    }

    pub trait Input {
        fn build(name: String, command: String) -> Result<InputStream, LogriaError>;
    }

    #[derive(Debug)]
    pub struct FileInput {}

    impl Input for FileInput {
        /// Create a file input
        /// poll_rate is unused since the file will be read all at once
        fn build(name: String, command: String) -> Result<InputStream, LogriaError> {
            // Setup multiprocessing queues
            let (_, err_rx) = channel();
            let (out_tx, out_rx) = channel();

            // Try and open a handle to the file
            // Remove, as file input should be immediately buffered...
            let path = Path::new(&command);
            // Ensure file exists
            let file = match File::open(&path) {
                // The `description` method of `io::Error` returns a string that describes the error
                Err(why) => {
                    return Err(LogriaError::CannotRead(
                        command,
                        <dyn Error>::to_string(&why),
                    ))
                }
                Ok(file) => file,
            };

            // Start process
            let process = thread::Builder::new()
                .name(format!("FileInput: {}", name))
                .spawn(move || {
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

            Ok(InputStream {
                stdout: out_rx,
                stderr: err_rx,
                process_name: name,
                process,
                _type: String::from("FileInput"),
            })
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
        fn build(name: String, command: String) -> Result<InputStream, LogriaError> {
            // Setup multiprocessing queues
            let (err_tx, err_rx) = channel();
            let (out_tx, out_rx) = channel();

            // Handle poll rate
            let mut poll_rate = RollingMean::new(5);

            // Start reading from the queues
            let process = thread::Builder::new()
                .name(format!("CommandInput: {}", name))
                .spawn(move || {
                    let runtime = Runtime::new().unwrap();
                    runtime.block_on(async {
                        let command_to_run = CommandInput::parse_command(&command);
                        let mut proc_read = match Command::new(command_to_run[0])
                            .args(&command_to_run[1..])
                            .current_dir(current_dir().unwrap())
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
                            thread::sleep(time::Duration::from_millis(poll_rate.mean()));

                            let timestamp = time::Instant::now();
                            let mut counter = 0;

                            loop {
                                tokio::select! {
                                    Ok(line) = stdout.next_line() => {
                                        if let Some(l) = line {
                                            out_tx.send(l).unwrap();
                                            counter += 1;
                                        } else { break }
                                    }
                                    Ok(line) = stderr.next_line() => {
                                        if let Some(l) = line {
                                            err_tx.send(l).unwrap();
                                            counter += 1;
                                        } else { break }
                                    }
                                    else => break
                                }
                            }

                            poll_rate.update(ms_per_message(timestamp.elapsed(), counter));
                        }
                    });
                });

            Ok(InputStream {
                stdout: out_rx,
                stderr: err_rx,
                process_name: name,
                process,
                _type: String::from("CommandInput"),
            })
        }
    }
}

fn determine_stream_type(command: &str) -> SessionType {
    let path = Path::new(command);
    match path.exists() {
        true => match is_executable(path) {
            true => SessionType::Command,
            false => SessionType::File,
        },
        false => SessionType::Command,
    }
}

/// Build app streams from user input, i.e. command text or a filepath
pub fn build_streams_from_input(
    commands: &[String],
    save: bool,
) -> Result<Vec<InputStream>, LogriaError> {
    let mut streams: Vec<InputStream> = vec![];
    let mut stream_types: HashSet<SessionType> = HashSet::new();
    for command in commands {
        // Determine if command is a file, create FileInput if it is, CommandInput if not
        match determine_stream_type(command) {
            SessionType::Command => {
                // None indicates default poll rate
                match CommandInput::build(command.to_owned(), command.to_owned()) {
                    Ok(stream) => streams.push(stream),
                    Err(why) => return Err(why),
                };
                stream_types.insert(SessionType::File);
            }
            SessionType::File => {
                // None indicates default poll rate
                let path = Path::new(command);
                let name = path.file_name().unwrap().to_str().unwrap().to_string();
                match FileInput::build(name, command.to_owned()) {
                    Ok(stream) => streams.push(stream),
                    Err(why) => return Err(why),
                };
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
        return match Session::new(commands, stream_type).save(&commands[0]) {
            Ok(_) => Ok(streams),
            Err(why) => Err(why),
        };
    }
    Ok(streams)
}

/// Build app streams from a session struct
pub fn build_streams_from_session(session: Session) -> Result<Vec<InputStream>, LogriaError> {
    match session.stream_type {
        SessionType::Command => {
            let mut streams: Vec<InputStream> = vec![];
            for command in session.commands {
                match CommandInput::build(command.to_owned(), command.to_owned()) {
                    Ok(stream) => streams.push(stream),
                    Err(why) => return Err(why),
                };
            }
            Ok(streams)
        }
        SessionType::File => {
            let mut streams: Vec<InputStream> = vec![];
            for command in session.commands {
                match FileInput::build(command.to_owned(), command.to_owned()) {
                    Ok(stream) => streams.push(stream),
                    Err(why) => return Err(why),
                };
            }
            Ok(streams)
        }
        SessionType::Mixed => build_streams_from_input(&session.commands, false),
    }
}

pub mod input_type {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum InputType {
        Normal,
        Command,
        Regex,
        Parser,
        Startup,
    }
}

pub mod stream_type {
    #[derive(Debug, Clone, Copy)]
    pub enum StreamType {
        StdErr,
        StdOut,
        Auxiliary,
    }
}

#[cfg(test)]
mod session_type_tests {
    use crate::{communication::input::determine_stream_type, extensions::session::SessionType};

    #[test]
    fn can_build_command_simple() {
        assert_eq!(determine_stream_type("ls"), SessionType::Command);
    }

    #[test]
    fn can_build_command_simple_arg() {
        assert_eq!(determine_stream_type("ls -lga"), SessionType::Command);
    }

    #[test]
    fn can_build_command_simple_pipe() {
        assert_eq!(
            determine_stream_type("echo 'thing' | cat"),
            SessionType::Command
        );
    }

    #[test]
    fn can_build_command_awk_file() {
        assert_eq!(
            determine_stream_type(
                "awk '{ if (length($0) > max) max = length($0) } END { print max }' fake.txt"
            ),
            SessionType::Command
        );
    }

    #[test]
    fn can_build_command_qualified_path_args() {
        assert_eq!(
            determine_stream_type("/bin/cp fake.txt fake2.txt"),
            SessionType::Command
        );
    }

    #[test]
    fn can_build_command_qualified_path_no_args() {
        assert_eq!(determine_stream_type("/bin/pwd"), SessionType::Command);
    }

    #[test]
    fn can_build_file_simple() {
        assert_eq!(determine_stream_type("/"), SessionType::File);
    }
}

#[cfg(test)]
mod stream_tests {
    use crate::{
        communication::input::{build_streams_from_input, build_streams_from_session},
        extensions::session::{Session, SessionType},
    };

    #[test]
    fn test_build_file_stream() {
        let commands = vec![String::from("README.md")];
        let streams = build_streams_from_input(&commands, false).unwrap();
        assert_eq!(streams[0]._type, "FileInput");
    }

    #[test]
    fn test_build_command_stream() {
        let commands = vec![String::from("ls -la ~")];
        let streams = build_streams_from_input(&commands, false).unwrap();
        assert_eq!(streams[0]._type, "CommandInput");
    }

    #[test]
    fn test_build_command_and_file_streams() {
        let commands = vec![String::from("ls -la ~"), String::from("README.md")];
        let streams = build_streams_from_input(&commands, false).unwrap();
        assert_eq!(streams[0]._type, "CommandInput");
        assert_eq!(streams[1]._type, "FileInput");
    }

    #[test]
    fn test_build_multiple_command_streams() {
        let commands = vec![String::from("ls -la ~"), String::from("ls /")];
        let streams = build_streams_from_input(&commands, false).unwrap();
        assert_eq!(streams[0]._type, "CommandInput");
        assert_eq!(streams[1]._type, "CommandInput");
    }

    #[test]
    fn test_build_multiple_file_streams() {
        let commands = vec![String::from("README.md"), String::from("Cargo.toml")];
        let streams = build_streams_from_input(&commands, false).unwrap();
        assert_eq!(streams[0]._type, "FileInput");
        assert_eq!(streams[1]._type, "FileInput");
    }

    #[test]
    fn test_build_file_stream_from_session() {
        let session = Session::new(&[String::from("README.md")], SessionType::File);
        let streams = build_streams_from_session(session).unwrap();
        assert_eq!(streams[0]._type, "FileInput");
    }

    #[test]
    fn test_build_command_stream_from_session() {
        let session = Session::new(&[String::from("ls -l")], SessionType::Command);
        let streams = build_streams_from_session(session).unwrap();
        assert_eq!(streams[0]._type, "CommandInput");
    }

    #[test]
    fn test_build_mixed_stream_from_session() {
        let session = Session::new(
            &[String::from("ls -l"), String::from("README.md")],
            SessionType::Mixed,
        );
        let streams = build_streams_from_session(session).unwrap();
        assert_eq!(streams[0]._type, "CommandInput");
        assert_eq!(streams[1]._type, "FileInput");
    }
}
