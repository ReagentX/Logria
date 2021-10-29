![Logria Logo](/resources/branding/logria.svg)

# Logria

A powerful CLI tool that puts log aggregation at your fingertips.

## tl;dr

- Live filtering/parsing of data from other processes
- Use shell commands or files as input, save sessions and come back later
- Replace regex/filter without killing the process or losing the stream's history
- Parse logs using user-defined rules, apply aggregation methods on top

## Installation

There are several options to install this app.

### Normal Usage

`cargo install logria` (not supported in alpha stage)

### Development

See [Advanced Installation](docs/README.md#advanced-installation).

## Usage

There are a few main ways to invoke Logria:

- Directly:
  - `logria`
  - Opens to the setup screen
- With args:
  - `logria -e 'tail -f log.txt'`
  - Opens a pipe to `tail -f log.txt` and skips setup
  - `logria -h` will show the help page with all possible options

For more details, see [Sample Usage Session](docs/README.md#sample-usage-session).

## Key Commands

| Key | Command |
|--|--|
| `:` | [command mode](docs/commands.md) |
| `/` | regex search |
| `h` | if regex active, toggle highlighting of matches |
| `s` | swap reading `stderr` and `stdout` |
| `p` | activate parser |
| `a` | toggle aggregation mode when parser is active |
| `z` | deactivate parser |
| ↑ | scroll buffer up one line |
| ↓ | scroll buffer down one line |
| → | skip and stick to end of buffer |
| ← | skip and stick to beginning of buffer |

## Features

Here are some of the ways you can leverage Logria:

### Live stream of log data

![logria](/resources/screenshots/logria.png)

### Interactive, live, editable regex search

![regex](/resources/screenshots/regex.png)

### Live log message parsing

![parser](/resources/screenshots/parser.png)

### Live aggregation/statistics tracking

![aggregation](/resources/screenshots/aggregation.png)

### User-defined saved sessions

See [session](/docs/sessions.md) docs.

### User-defined saved log parsing methods

See [patterns](/docs/patterns.md) docs.

## Notes

This is a Rust implementation of my [Python](https://github.com/ReagentX/Logria-py) proof-of-concept.

### What is Logria For

Logria is best leveraged to watch live logs from multiple processes and filter them for events you want to see. My most common use case is watching logs from multiple Linode/EC2 instances via `ssh` or multiple CloudWatch streams using [`awslogs`](https://github.com/jorgebastida/awslogs).

I also use it to analyze the logs from my Apache web servers that print logs in the common log format.

### What is Logria Not For

Logria is not a tool for detailed log analytics. [`lnav`](https://lnav.org/features) or [`angle-grinder`](https://github.com/rcoh/angle-grinder/) will both do the job better.

## Special Thanks

- [Voidsphere](https://voidsphere.bandcamp.com), for providing all the hacking music I could want.
- [Julian Coleman](https://github.com/juliancoleman/), for lots of code review and general Rust advice.
- [@rhamorim](https://twitter.com/rhamorim), for [suggesting](https://twitter.com/rhamorim/status/1333856615624306692) an alternative for non-blocking IO without `O_NONBLOCK`.
- [@andy_crab_gear](https://twitter.com/andy_crab_gear), for [suggesting](https://twitter.com/andy_crab_gear/status/1333866079555239936) an alternative for non-blocking IO without `O_NONBLOCK`.
- [yonkeltron](https://github.com/yonkeltron), for advice and help learning Rust.
- [Simone Vittori](https://www.simonewebdesign.it), for a great [blog post](https://www.simonewebdesign.it/rust-hashmap-insert-values-multiple-types/) on storing multiple value types in a `HashMap`.
