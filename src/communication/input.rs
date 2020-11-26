pub mod stream {
    use std::error::Error;
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::path::Path;
    use std::sync::mpsc::{channel, Receiver};
    use std::sync::{Arc, Mutex};
    use std::thread;

    use crate::constants::cli::poll_rate::FASTEST;

    #[derive(Debug)]
    pub struct InputStream {
        pub poll_rate: Arc<Mutex<u64>>,
        pub stdout: Receiver<String>,
        pub stderr: Receiver<String>,
        pub proccess_name: String,
        pub process: Result<std::thread::JoinHandle<()>, std::io::Error>,
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
            out_tx
                .send(String::from("No stderr for File Input!"))
                .unwrap(); // Otherwise typing breaks

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
                        out_tx.send(line.unwrap()).unwrap();
                    }
                });

            InputStream {
                poll_rate: poll_rate,
                stdout: out_rx,
                stderr: err_rx,
                proccess_name: name,
                process: process,
            }
        }
    }

    #[derive(Debug)]
    pub struct CommandInput {}

    impl CommandInput {
        fn resolve_command(&self, command: &str) -> Vec<String> {
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
                .spawn(move || {}); // TODO

            InputStream {
                poll_rate: poll_rate,
                stdout: out_rx,
                stderr: err_rx,
                proccess_name: name,
                process: process,
            }
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
