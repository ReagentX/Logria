# Commands

| Key | Command |
|--|--|
| `:` | enter command mode |
| `:q` | exit Logria |
| `:poll #` | update [poll rate](#poll-rate) to #, where # is an integer |
| `:r #` | when launching logria or viewing sessions, this will delete item # |
| `:restart` | go back to the setup screen to change sessions |
| `:agg #` | set the limit for aggregation counters be `top #`, i.e. `top 5` or `top 1` |

## Notes

To use a command, simply type `:` and enter a command. To exit without running the command, press `esc`.

### Poll Rate

This is the rate at which Logria checks the queues for new messages.

The poll rate defaults to `smart` mode, where Logria will calculate a rate at which to poll the message queues based on the speed of incoming messages. To disable this feature, pass `-n` when starting Logria. If `smart` mode is disabled, the app falls back to the default value of polling once every `50` milliseconds.

### Remove Command

The command `:r` is applicable only when the user is loading either sessions or parsers. `:r 2` will remove item 2, `:r 0-4` will remove items 0 through 4 inclusively. Any combination of those two patterns will work: for example, `:r 2,4-6,8` will remove 2, 4, 5, 6, and 8.
