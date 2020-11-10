# Sessions Documentation

A session is a collection of commands that result in input streams.

## Storage

Sessions are stored as `JSON` in `~/.logria/sessions` and do not have file extensions. A pattern is defined like so:

```json
{
    "commands": [
        [
            "/bin/python",
            "sample_streams/generate_test_logs.py"
        ],
        [
            "/bin/python",
            "sample_streams/generate_test_logs_2.py"
        ]
    ],
    "type": "command"
}
```

If `~/.logria/sessions` does not exist, Logira will create it.

## Elements

All sessions have two keys:

- `commands`
  - Contains a list of commands to listen on
- `type`
  - Contains a string of the type of input handler to use, either `file` or `command`
  - `file` creates a `FileInputHander` and `command` creates a `CommandInputHandler`

## Interpreting Sessions at Runtime

If Logria is launched without `-e`, it will default to listing the contents of `~/.logria/sessions` and allow the user to select one. Users can also enter a new command to follow, and that command will be saved as a new session.

```zsh
  Enter a new command to open a stream or choose a saved one from the list:
  0: Generate Test Logs
  1: Tail Logfile
```

Once a selection has been made, Logria will open pipes to the new processes and begin streaming.

```zsh
  2020-02-08 19:00:02,317 - __main__.<module> - MainProcess - INFO - I am a first log! 80
  2020-02-08 19:00:02,317 - __main__.<module> - MainProcess - INFO - I am a second log! 43
  2020-02-08 19:00:02,327 - __main__.<module> - MainProcess - INFO - I am a first log! 80
  2020-02-08 19:00:02,327 - __main__.<module> - MainProcess - INFO - I am a second log! 58
  2020-02-08 19:00:02,337 - __main__.<module> - MainProcess - INFO - I am a second log! 54
  2020-02-08 19:00:02,337 - __main__.<module> - MainProcess - INFO - I am a first log! 92
  2020-02-08 19:00:02,347 - __main__.<module> - MainProcess - INFO - I am a second log! 68
  2020-02-08 19:00:02,350 - __main__.<module> - MainProcess - INFO - I am a first log! 26
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│No filter applied
└─────────────────────────────────────────────────────────────────────────────────────────┘
```
