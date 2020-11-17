pub mod stream {
    use std::sync::mpsc::{channel, Receiver};
    use std::io::{BufRead, BufReader};
    use std::sync::{Arc, Mutex};
    use std::error::Error;
    use std::path::Path;
    use std::fs::File;
    use std::thread;

    use crate::constants::cli::poll_rate::FASTEST;

    #[derive(Debug)]
    pub struct InputStream {
        pub poll_rate: Arc<Mutex<u64>>,
        pub stdout: Receiver<&'static str>,
        pub stderr: Receiver<String>,
        pub proccess_name: String,
        pub process: Result<std::thread::JoinHandle<()>, std::io::Error>,
    }

    #[derive(Debug)]
    pub struct FileInput {}

    impl FileInput {
        pub fn new(poll_rate: Option<u64>, name: String, command: String) -> InputStream {
            // Setup multiprocessing queues
            let (err_tx, err_rx) = channel();
            let (out_tx, out_rx) = channel();
            out_tx.send("").unwrap(); // Same as above

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
                        err_tx.send(line.unwrap().to_string()).unwrap();
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
}
