# Logria Documentation

This folder contains the documentation on how to interact with Logria programmatically as well as how to leverage its feature-set as a user.

## Index

- [Parsers](parsers.md)
  - Details on how to configure parsers for log parsing
- [Sessions](sessions.md)
  - Details on how to configure sessions when launching the app
- [Input Handler](input_handler.md)
  - Details on how input handler classes open subprocesses
- [Commands](commands.md)
  - Details on commands available in the app

## Advanced Installation

`cargo install logria` is the best way to install the app for normal use.

### Installing as a standalone app

- `clone` the repository
- `cd` to the repository
- `cargo test` to make sure everything works
- `cargo run --release` to compile

### Directory Configuration

By default, Logria will create a `/Logria/` directory to store [parsers](parsers.md), [sessions](sessions.md), and an input history tape in. The location is [platform dependent](https://docs.rs/dirs/latest/dirs/fn.config_dir.html).

#### Platform-Specific Directory Locations

| Platform | Value | Example |
| --- | --- | --- |
| Linux   | `$XDG_DATA_HOME` or `$HOME/.config` | `~/.config/Logria` |
| macOS   | `$HOME/Library/Application Support` | `~/Library/Application Support/Logria` |
| Windows | `{FOLDERID_RoamingAppData}` | `%homedrive%%homepath%\AppData\Roaming\Logria` |

If you want to specify a different path, either set `LOGRIA_ROOT` to replace the `/Logria/` directory or set `LOGRIA_USER_HOME` to move the directory away from the default `$HOME`. Setting both means the app looks in `$LOGRIA_USER_HOME/$LOGRIA_ROOT`.

#### Example Exports

| Environment Variable | Value | Result |
|---|---|---|
| `LOGRIA_ROOT` | `.conf/.logria` | `~/.conf/.logria/` |
| `LOGRIA_USER_HOME` | `/usr/local/` | `/usr/local/Logria` |
| both of the above | | `/usr/local/.conf/.logria/` |

## Sample Usage Session

To see available commands, invoke Logria with `-h`:

```zsh
chris@home ~ % logria -h
A powerful CLI tool that puts log analytics at your fingertips.

USAGE:
    logria [FLAGS] [OPTIONS]

Usage: logria [OPTIONS]

Options:
  -t, --no-history-tape  Disable command history disk cache
  -m, --mindless         Disable variable polling rate based on incoming message rate
  -d, --docs             Prints documentation
  -p, --paths            Prints current configuration paths
  -e, --exec <stream>    Command to listen to, ex: logria -e "tail -f log.txt"
  -h, --help             Print help information
  -V, --version          Print version information
```

Start Logria by invoking it as a command line application:

```zsh
chris@home ~ % logria
```

This will launch the app and show us the splash screen:

```log
Enter a new command to open and save a new stream,
or enter a number to choose a saved session from the list.

Enter `:r #` to remove session #.
Enter `:q` to quit.

0: File - readme
1: File - Sample Access Log
2: Cmd - Generate Test Logs
┌────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                                │
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
│                                                                                                │
└────────────────────────────────────────────────────────────────────────────────────────────────┘
```

Typing `/` and entering `100` will filter our stream down to only lines that match that pattern:

```log
2020-02-23 16:56:11,100 - __main__.<module> - MainProcess - INFO - I am a first log! 43
2020-02-23 16:56:11,100 - __main__.<module> - MainProcess - INFO - I am a second log! 87
┌────────────────────────────────────────────────────────────────────────────────────────────────┐
│Regex with pattern /100/                                                                        │
└────────────────────────────────────────────────────────────────────────────────────────────────┘
```

Pressing `esc` will reset the filter:

```log
2020-02-23 16:56:10,786 - __main__.<module> - MainProcess - INFO - I am the first log in the list
2020-02-23 16:56:10,997 - __main__.<module> - MainProcess - INFO - I am a first log! 21
2020-02-23 16:56:10,997 - __main__.<module> - MainProcess - INFO - I am a second log! 71
2020-02-23 16:56:11,100 - __main__.<module> - MainProcess - INFO - I am a first log! 43
2020-02-23 16:56:11,100 - __main__.<module> - MainProcess - INFO - I am a second log! 87
┌────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                                │
└────────────────────────────────────────────────────────────────────────────────────────────────┘
```

Typing `:` and entering `:q` will exit the app.

## Contributing Guidelines

- No pull request shall be behind develop
- First come, first served
- If anything breaks, the pull request will be queued again when the issue is resolved
- Pull request comments will be resolved by the person who created them

## Logo Colors

- Letters: ![#e63462](../resources/img/e63462.png) `#e63462`
- Accent: ![#333745](../resources/img/333745.png) `#333745`

## Notes / Caveats

- When using `tmux` or other emulators that change the `$TERM` environment variable, you must set the default terminal to something that supports color. In `tmux`, this is as simple as adding `set -g default-terminal "screen-256color"` to `.tmux.conf`.
- The package version in `Cargo.toml` is `0.0.0`. This is because during CICD that value gets [replaced](/.github/workflows/release.yml) with the current release tag name.
