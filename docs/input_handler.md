# Input Handler Documentation

Input handlers run in processes parallel to the main process using Python's `multiprocessing` library. Each `InputStream` child class implements a method that creates two pipes, one for `stdin` and one for `stdout`.  The data sent through these pipes are stored in a Queue, which the main process can read from to render.

## `CommandInputStream` Objects

Given a list command parts, use the `subprocess` library to open a shell, run that process, and pipe the responses back into their respective queues.

Commands will be parsed against your PATH variables, replacing programs you have on your path with their fully qualified path.

Creating a `CommandInputStream()` with `args` like `['tail', '-f', 'out.log']` will open a shell that runs `/usr/bin/tail -f out.log`.

## `FileInputStream` Objects

Given a list that represents a file path, read in the file and send the output to the `stdout` queue.

Creating a `FileInputStream()` with `args` like `["sample_streams", "accesslog"]` will read in the contents of `sample_streams/accesslog` to the `stdout` queue.
