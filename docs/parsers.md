# Parser Documentation

A parser includes a pattern with associated metadata that Logria uses to parse and aggregate log messages.

## Storage

Parsers are stored as `JSON` in `$LOGRIA_USER_HOME/$LOGRIA_ROOT/parsers` and do not have file extensions. A parser is defined like so:

```json
{
    "pattern": " - ",
    "pattern_type": "Split",
    "example": "2005-03-19 15:10:26,773 - simple_example - CRITICAL - critical message",
    "order": [
      "Timestamp",
      "Method",
      "Level",
      "Message"
    ],
    "aggregation_methods": {
        "Timestamp": {
            "DateTime": "[year]-[month]-[day] [hour]:[minute]:[second],[subsecond]"
        },
        "Method": "Count",
        "Level": "Count",
        "Message": "Sum"
    }
}
```

If `$LOGRIA_USER_HOME/$LOGRIA_ROOT/parsers` does not exist, Logria will create it.

## Anatomy

All parsers have the following keys:

- `pattern`
  - The pattern to apply
- `pattern_type`
  - The method we intend to apply the pattern with, one of {`regex`, `split`}, detailed in [Types of Parsers](#types-of-parsers)
- `name`
  - The name of the parser
  - Displayed to the user when selecting parsers
- `example`
  - An example message to match with the parser
  - Displayed to the user when selecting which part of a message to render
- `order`
  - The order the message parts occur in for aggregation
- `aggregation_methods`
  - Can be `Mean`, `Sum`, `Count`, `Mode`, `Date`, `Time`, `DateTime`, and `None`
  - See [Aggregation Methods](#aggregation-methods) below for details

## Types of Parsers

There are two types of parsers: `regex` and `split`.

### Regex Parser

A `regex` parser uses a regex expression to match parts of a log and looks like this:

```json
{
    "pattern": "([^ ]*) ([^ ]*) ([^ ]*) \\[([^]]*)\\] \"([^\"]*)\" ([^ ]*) ([^ ]*)",
    "pattern_type": "Regex",
    "example": "127.0.0.1 user-identifier user-name [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326",
    "order": [
        "Remote Host",
        "User ID",
        "Username",
        "Date",
        "Request",
        "Status",
        "Size",
    ],
    "aggregation_methods": {
        "Remote Host": "Count",
        "User ID": "Count",
        "Username": "Count",
        "Date": "Count",
        "Request": "Count",
        "Status": "Count",
        "Size": "Count"
    }
}
```

### Split Patterns

A `split` parser uses [str::split](https://doc.rust-lang.org/std/primitive.str.html#method.split) to split a message on a delimiter:

```json
{
    "pattern": " - ",
    "pattern_type": "Split",
    "example": "2005-03-19 15:10:26,773 - simple_example - CRITICAL - critical message",
    "order": [
        "Timestamp",
        "Method",
        "Level",
        "Message"
    ],
    "aggregation_methods": {
        "Timestamp": {
            "DateTime": "[year]-[month]-[day] [hour]:[minute]:[second],[subsecond]"
        },
        "Method": "Count",
        "Level": "Count",
        "Message": "Sum"
    }
}
```

## Aggregation Methods

The `aggregation_methods` key stores a `HashMap<String, AggregationMethod>` of the name of the parsed message to a method to handle message aggregation. Since `HashMap`s are unordered, a list called `order` must also be present. This list contains strings that match the key names in `aggregation_methods`.

### Included Methods

Methods currently include [`Mean`](#mean-and-sum), [`Sum`](#mean-and-sum), [`Count`](#count-and-mode),[`Mode`](#count-and-mode) [`Date`](#date-time-and-datetime), [`Time`](#date-time-and-datetime), [`DateTime`](#date-time-and-datetime), and [`None`](#none). These all have different behaviors.

#### Mean and Sum

Both of these methods search for the first occurance of a number in the parsed message.

`Mean` will display the mean, the count, and the sum of the parsed floats:

```txt
Message
    Mean: 51.32
    Count: 5.113
    Total: 262,417
```

`Sum` will only display the sum:

```txt
Message
    Total: 262,417
```

Float parsing logic is defined and tested in [aggregators.rs](../src/util/aggregators/aggregator.rs). Some examples include:

```rust
extract_number("653.12 this is a test");  // 653.12
extract_number("4.123 this is a test 123.4");  // 4.123
extract_number("this is a 123.123. test");  // None, invalid
```

#### Count and Mode

This uses a data structure similar to Python's [`collections.Counter`](https://docs.python.org/3/library/collections.html#collections.Counter) to keep track of messages. Each message is hashed, so identical messages will get incremented. It defaults to displaying the top 5 results; this can be adjusted using the `:agg` [command](commands.md#commands).

When activated, it will display the ordinal count of each occurance as well as its ratio to the total amount of messages counted:

```txt
Level
    INFO: 2,794 (55%)
    WARNING: 1,433 (28%)
    ERROR: 886 (17%)
```

`Mode` is a special case of `Couter` where the top `n` is frozen to `1`.

#### Date, Time, and DateTime

`Date`, `Time`, or `DateTime` methods require a format description as outlined in the [`time` book](https://time-rs.github.io/book/api/format-description.html) or [`time` docs](https://docs.rs/time/0.3.3/time/struct.Date.html#method.parse).

`Date` will default all messages to [midnight](https://docs.rs/time/latest/time/struct.Time.html#associatedconstant.MIDNIGHT) and `Time` will default all messages to [min](https://docs.rs/time/latest/time/struct.Date.html#associatedconstant.MIN).

When activated, these methds display the rate at which messages are received, the total number of messgaes, and the earliest and latest timestamps.

```txt
Timestamp
    Rate: 196 per second
    Count: 5,113
    Earliest: 2021-11-15 22:28:42.21
    Latest: 2021-11-15 22:29:08.389
```

#### None

`None` disables parsing for that field. It displays like this when activated:

```txt
Timestamp
    Disabled
```

### Example Aggregation Data

Given an `order` and `aggregation_map` with methods like this:

```json
"order": [
    "Timestamp",
    "Method",
    "Level",
    "Message"
],
"aggregation_methods": {
    "Timestamp": {
        "DateTime": "[year]-[month]-[day] [hour]:[minute]:[second],[subsecond]"
    },
    "Method": "Count",
    "Level": "Count",
    "Message": "Mean"
}
```

The resultant aggregation data will render like so:

```txt
Timestamp
    Rate: 196 per second
    Count: 5,113
    Earliest: 2021-11-15 22:28:42.21
    Latest: 2021-11-15 22:29:08.389
Method
    __main__.‹module>: 2,215 (43%)
    __main__.first: 1,433 (28%)
    __main__.second: 886 (17%)
    __main__.third: 579 (11%)
Process
    MainProcess: 5,113 (100%)
Level
    INFO: 2,794 (55%)
    WARNING: 1,433 (28%)
    ERROR: 886 (17%)
Message
    Mean: 51.32
    Count: 5.113
    Total: 262,417
┌────────────────────────────────────────────────────────────────────────────────────────────────┐
|Parsing with Color + Hyphen Separated, aggregation mode                                         │
└────────────────────────────────────────────────────────────────────────────────────────────────┘
```

## Activating Parsers

When invoked, Logria will list the parsers defined in the parsers directory for the user to select based on the index of the filename:

```zsh
  0: Common Log Format
  1: Hyphen Separated
  2: Color + Hyphen Separated
```

Once the first selection has been made, the user will be able to select which part of the matched log we will use when streaming:

```zsh
  0: 2020-02-04 19:06:52,852
  1: __main__.<module>
  2: MainProcess
  3: INFO
  4: I am a log! 91
```

This text is generated by the `example` key in the parser's `JSON`.
