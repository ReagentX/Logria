use is_executable::is_executable;

use crate::{
    extensions::{
        extension::ExtensionMethods,
        session::{Session, SessionType},
    },
    util::{
        error::LogriaError,
        poll::{RollingMean, ms_per_message},
    },
};

use std::{
    collections::HashSet,
    env::current_dir,
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    process::{Command, Stdio},
    result::Result,
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, channel},
    },
    thread, time,
};

#[derive(Debug)]
pub struct InputStream {
    pub stdout: Receiver<String>,
    pub stderr: Receiver<String>,
    pub should_die: Arc<Mutex<bool>>,
    pub _type: String,
}

pub trait Input {
    fn build(name: String, command: String) -> Result<InputStream, LogriaError>;
}

#[derive(Debug)]
pub struct FileInput {}

impl Input for FileInput {
    /// Create a file input
    /// `poll_rate` is unused since the file will be read all at once
    fn build(name: String, command: String) -> Result<InputStream, LogriaError> {
        // Setup multiprocessing queues
        let (_, err_rx) = channel();
        let (out_tx, out_rx) = channel();

        // Try and open a handle to the file
        // Remove, as file input should be immediately buffered...
        let path = Path::new(&command);
        // Ensure file exists
        let file = match File::open(path) {
            // The `description` method of `io::Error` returns a string that describes the error
            Err(why) => {
                return Err(LogriaError::CannotRead(
                    command,
                    <dyn Error>::to_string(&why),
                ));
            }
            Ok(file) => file,
        };

        // Start process
        let _ = thread::Builder::new()
            .name(format!("FileInput: {name}"))
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
            should_die: Arc::new(Mutex::new(false)),
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

        // Provide check for termination outside of the thread
        let should_die = Arc::new(Mutex::new(false));
        let die = should_die.clone();

        // Handle poll rate for each stream
        let poll_rate_stdout = Arc::new(Mutex::new(RollingMean::new(5)));
        let poll_rate_stderr = Arc::new(Mutex::new(RollingMean::new(5)));

        // Start reading from the queues
        let _ = thread::Builder::new()
            .name(format!("CommandInput: {name}"))
            .spawn(move || {
                let command_to_run = CommandInput::parse_command(&command);
                let mut child = match Command::new(command_to_run[0])
                    .args(&command_to_run[1..])
                    .current_dir(current_dir().unwrap())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .stdin(Stdio::null())
                    .spawn()
                {
                    Ok(child) => child,
                    Err(why) => panic!("Unable to connect to process: {why}"),
                };

                // Get stdout and stderr handles
                let stdout = child.stdout.take().unwrap();
                let stderr = child.stderr.take().unwrap();

                // Create readers
                let mut stdout_reader = BufReader::new(stdout);
                let mut stderr_reader = BufReader::new(stderr);

                // Create threads to read stdout and stderr independently
                let die_clone = die.clone();
                let poll_stdout = poll_rate_stdout.clone();
                let stdout_handle = thread::spawn(move || {
                    loop {
                        thread::sleep(time::Duration::from_millis(
                            poll_stdout.lock().unwrap().mean(),
                        ));

                        let mut buf_stdout = String::new();
                        let timestamp = time::Instant::now();
                        stdout_reader.read_line(&mut buf_stdout).unwrap();

                        if buf_stdout.is_empty() {
                            poll_stdout
                                .lock()
                                .unwrap()
                                .update(ms_per_message(timestamp.elapsed(), 0));
                            continue;
                        }

                        if out_tx.send(buf_stdout).is_err() {
                            break;
                        }

                        poll_stdout
                            .lock()
                            .unwrap()
                            .update(ms_per_message(timestamp.elapsed(), 1));

                        if *die_clone.lock().unwrap() {
                            break;
                        }
                    }
                });

                let die_clone = die.clone();
                let poll_stderr = poll_rate_stderr.clone();
                let stderr_handle = thread::spawn(move || {
                    loop {
                        thread::sleep(time::Duration::from_millis(
                            poll_stderr.lock().unwrap().mean(),
                        ));

                        let mut buf_stderr = String::new();
                        let timestamp = time::Instant::now();
                        stderr_reader.read_line(&mut buf_stderr).unwrap();

                        if buf_stderr.is_empty() {
                            poll_stderr
                                .lock()
                                .unwrap()
                                .update(ms_per_message(timestamp.elapsed(), 0));
                            continue;
                        }

                        if err_tx.send(buf_stderr).is_err() {
                            break;
                        }

                        poll_stderr
                            .lock()
                            .unwrap()
                            .update(ms_per_message(timestamp.elapsed(), 1));

                        if *die_clone.lock().unwrap() {
                            break;
                        }
                    }
                });

                // Wait for both readers to complete
                stdout_handle.join().unwrap();
                stderr_handle.join().unwrap();

                // Kill the child process if requested
                if *die.lock().unwrap() {
                    let _ = child.kill();
                }
                let _ = child.wait();
            });

        Ok(InputStream {
            stdout: out_rx,
            stderr: err_rx,
            should_die,
            _type: String::from("CommandInput"),
        })
    }
}

fn determine_stream_type(command: &str) -> SessionType {
    let path = Path::new(command);
    if path.exists() {
        if is_executable(path) {
            SessionType::Command
        } else {
            SessionType::File
        }
    } else {
        SessionType::Command
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
                }
                stream_types.insert(SessionType::Command);
            }
            SessionType::File => {
                // None indicates default poll rate
                let path = Path::new(command);
                let name = path.file_name().unwrap().to_str().unwrap().to_string();
                match FileInput::build(name, command.to_owned()) {
                    Ok(stream) => streams.push(stream),
                    Err(why) => return Err(why),
                }
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
            Ok(()) => Ok(streams),
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
                match CommandInput::build(command.clone(), command.clone()) {
                    Ok(stream) => streams.push(stream),
                    Err(why) => return Err(why),
                }
            }
            Ok(streams)
        }
        SessionType::File => {
            let mut streams: Vec<InputStream> = vec![];
            for command in session.commands {
                match FileInput::build(command.clone(), command.clone()) {
                    Ok(stream) => streams.push(stream),
                    Err(why) => return Err(why),
                }
            }
            Ok(streams)
        }
        SessionType::Mixed => build_streams_from_input(&session.commands, false),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputType {
    Normal,
    Command,
    Regex,
    Parser,
    Startup,
}

#[derive(Debug, Clone, Copy)]
pub enum StreamType {
    StdErr,
    StdOut,
    Auxiliary,
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
