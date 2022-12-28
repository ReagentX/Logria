# Input Handler Documentation

Input handlers run in processes parallel to the main process and communicate using Rust's `mpsc` module. Each struct that implements the `Input` trait has a method that creates two sets of `mpsc` channels, one for `stdin` and one for `stdout`. The data sent through these channels are stored until the main process can read from them.

## `CommandInput`

Given a command, use `tokio`'s `command` module to open process and read its `stderr` and `stdout` into their respective `mpsc` channels.

## `FileInput`

Given a file path, read the file and send the output to the `stdout` queue.

Creating a `FileInput()` with `"sample_streams/accesslog"` will read in the contents of `sample_streams/accesslog` to the `stdout` queue. The path is parsed relative to the current directory when starting `logria`.
