pub mod stream {
    pub struct InputStream {
        poll_rate: f64,
        stdout: &'static str,  // change to mpsc
        stderr: &'static str,  // change to mpsc
        process: &'static str, // some sort of thread we can start
        proccess_name: String,
    }

    pub trait Communicate {
        fn start(&self);
        fn run(&self);
        fn exit(&self);
    }

    pub struct FileInput {
        pub stream: InputStream,
    }

    impl FileInput {
        pub fn new(poll_rate: Option<f64>, name: String) -> FileInput {
            FileInput {
                stream: InputStream {
                    poll_rate: poll_rate.unwrap_or(0.0001),
                    stdout: "",  // change to mpsc
                    stderr: "",  // change to mpsc
                    process: "", // some sort of thread we can start
                    proccess_name: name,
                },
            }
        }
    }

    impl Communicate for FileInput {
        fn start(&self) {
            println!("Started {}!", self.stream.proccess_name);
        }

        fn run(&self) {
            println!("Running {}!", self.stream.poll_rate);
        }

        fn exit(&self) {
            println!("Exited!")
        }
    }
}
