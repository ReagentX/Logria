# Logria Documentation

This folder contains the documentation on how to interact with Logria programmatically as well as how to leverage its feature-set as a user.

## Index

- [Patterns](patterns.md)
  - Details on how to configure patterns for log parsing
- [Sessions](sessions.md)
  - Details on how to configure sessions when launching the app
- [Input Handler](input_handler.md)
  - Details on how input handler classes open subprocesses
- [Commands](commands.md)
  - Details on commands available in the app
- [Todo](todo.md)
  - List of tasks for the repo

## Advanced Installation

`cargo install logria` is the best way to install the app for normal use.

### Installing as a standalone app

- `clone` the repository
- `cd` to the repository
- `cargo test` to make sure everything works
- `cargo run --release` to compile

### Configuration Directory

By default, Logria will create `~/.logria` to store configuration files in. If you want to specify a different path, either set `LOGRIA_ROOT` to replace the `.logria` directory or set `LOGRIA_DISABLE_USER_HOME` to move the directory away from the default `~`.

## Sample Usage Session

Start Logria by invoking it as a command line application:

```zsh
chris@ChristophersMBP ~ % logria
```

This will launch the app and show us the splash screen:

```log
Enter a new command to open and save a new stream,
or enter a number to choose a saved session from the list,
or enter `:config` to configure.
Enter `:q` to quit.

0: File - readme
1: File - Sample Access Log
2: Cmd - Generate Test Logs
┌────────────────────────────────────────────────────────────────────────────────────────────────┐
│_
└────────────────────────────────────────────────────────────────────────────────────────────────┘
```

Entering `2` will load and open handles to the commands in `Cmd - Generate Test Logs`:

```log
2020-02-23 16:56:10,786 - __main__.<module> - MainProcess - INFO - I am the first log in the list
2020-02-23 16:56:10,997 - __main__.<module> - MainProcess - INFO - I am a first log! 21
2020-02-23 16:56:10,997 - __main__.<module> - MainProcess - INFO - I am a second log! 71
2020-02-23 16:56:11,100 - __main__.<module> - MainProcess - INFO - I am a first log! 43
2020-02-23 16:56:11,100 - __main__.<module> - MainProcess - INFO - I am a second log! 87
┌────────────────────────────────────────────────────────────────────────────────────────────────┐
│No filter applied
└────────────────────────────────────────────────────────────────────────────────────────────────┘
```

Typing `/` and entering `100` will filter our stream down to only lines that match that pattern:

```log
2020-02-23 16:56:11,100 - __main__.<module> - MainProcess - INFO - I am a first log! 43
2020-02-23 16:56:11,100 - __main__.<module> - MainProcess - INFO - I am a second log! 87
┌────────────────────────────────────────────────────────────────────────────────────────────────┐
│Regex with pattern /100/
└────────────────────────────────────────────────────────────────────────────────────────────────┘
```

Typing `/` and entering `:q` will reset the filter:

```log
2020-02-23 16:56:10,786 - __main__.<module> - MainProcess - INFO - I am the first log in the list
2020-02-23 16:56:10,997 - __main__.<module> - MainProcess - INFO - I am a first log! 21
2020-02-23 16:56:10,997 - __main__.<module> - MainProcess - INFO - I am a second log! 71
2020-02-23 16:56:11,100 - __main__.<module> - MainProcess - INFO - I am a first log! 43
2020-02-23 16:56:11,100 - __main__.<module> - MainProcess - INFO - I am a second log! 87
┌────────────────────────────────────────────────────────────────────────────────────────────────┐
│No filter applied
└────────────────────────────────────────────────────────────────────────────────────────────────┘
```

Typing `/` and entering `:q` will reset the filter:

```log
2020-02-23 16:56:10,786 - __main__.<module> - MainProcess - INFO - I am the first log in the list
2020-02-23 16:56:10,997 - __main__.<module> - MainProcess - INFO - I am a first log! 21
2020-02-23 16:56:10,997 - __main__.<module> - MainProcess - INFO - I am a second log! 71
2020-02-23 16:56:11,100 - __main__.<module> - MainProcess - INFO - I am a first log! 43
2020-02-23 16:56:11,100 - __main__.<module> - MainProcess - INFO - I am a second log! 87
┌────────────────────────────────────────────────────────────────────────────────────────────────┐
│No filter applied
└────────────────────────────────────────────────────────────────────────────────────────────────┘
```

Typing `:` and entering `:q` will exit the app.

## Guidelines

- "Brand" colors
  - Letters: ![#e63462](https://placehold.it/15/e63462/000000?text=+)`#e63462`
  - Accent: ![#333745](https://placehold.it/15/333745/000000?text=+)`#333745`
- Contributing
  - No pull request shall be behind develop
  - First come, first served
  - If anything breaks, the pull request will be queued again when the issue is resolved
  - Pull request comments will be resolved by the person who created them

## Notes / Caveats

- Curses will crash when writing to the last line of a window, but it will write correctly, so we wrap some instances of this in a try/except to ensure we don't crash when writing valid values
- When using `tmux` or other emulators that change the `$TERM` environment variable, you must set the default terminal to something that supports color. In `tmux`, this is as simple as adding `set -g default-terminal "screen-256color"` to `.tmux.conf`.
