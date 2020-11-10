# Todo

- Enhancements
  - [ ] Esc key to go back to previous state
  - [ ] Spawn a subprocess to find all the matches in the list of messages
- Clerical
  - [ ] Deeper instructions for sessions and parsers
  - [ ] Update contribution guidelines with concrete rules about what PRs will be accepted

## Completed

- [x] Custom textbox implementation that respects poll_rate
- [x] "Configuration" mode or "setup" mode to generate and save sessions/parsers
- [x] Add example folder for sessions and parsers
- [x] Test suite
- [x] Write docs
- [x] Add contribution guidelines
- [x] Refactor command handlers that are > 5 lines to method calls
- [x] Support line breaks - requires rework of rendering logic
- [x] Support updating poll rate
- [x] Make the command line show what current command is active, ex `/` for regex mode, `:` for command, etc
- [x] Screenshots for readme
- [x] Add license
- [x] Add statistics tracking for log messages
- [x] Allow user to define multiple streams e.x. `ssh` sessions, and have a class to join them together
- [x] Main app loop starts when we call start, but the listener happens on init
- [x] Save sessions through class, make init process nicer
- [x] Init screen when launched with no args
- [x] Class for parsing paths for shell commands, i.e. resolving paths to tools on the `PATH`
- [x] Support parsing logs using `Log()` class
- [x] Switch between stderr and stdout
- [x] Move `regex_test_generator` to a separate class/module
- [x] Toggle insert mode (default off)
- [x] Add app entry method to `setup.py`
- [x] Highlight match in log - requires rework of regex method
  - We cannot just add ANSI codes as we might overwrite/alter existing ones
  - We also cannot just use a reset code after we insert a new code because it may reset what was already in the message
  - Current workaround is to regex out all color codes before inserting a highlighter and toggle
- [x] Regex searches through pre-formatted string, not color formatted string - requires rework of regex method
- [x] Make window scroll
- [x] Move with arrow keys
- [x] Refactor to class
- [x] Handle editor validation
- [x] Make backspace work

## Rejected

- [ ] Multiprocessing manager dict for `{stdout: [], stdin: []}`
  - This is not possible because to access the data in the array we must wait for the subprocess to complete, which defeats the purpose of this app.
- [ ] Support optional piping as input stream - [SO Link](https://stackoverflow.com/questions/1450393/how-do-you-read-from-stdin)
  - Not possible to implement as Logria requires the user to be in control of stdin
  - stdin gets taken over by whatever we pipe to this program, and we cannot move that pipe away from stdin until the pipe finishes
  - We can overwrite the pipe with `sys.stdin = open(0)` however this will not work until the original pipe ends, which will never happen when tailing a stream
