use regex::Error;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum LogriaError {
    InvalidRegex(Error, String),
    WrongParserType,
    InvalidExampleRegex(String),
    InvalidExampleSplit(usize, usize),
    CannotRead(String, String),
    CannotWrite(String, String),
    CannotRemove(String, String),
    CannotParseDate(String),
    InvalidCommand(String),
    CannotParseMessage(String),
}

impl Display for LogriaError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        match self {
            LogriaError::InvalidRegex(why, msg) => write!(fmt, "{}: {}", why, msg),
            LogriaError::WrongParserType => {
                write!(fmt, "Cannot construct regex for a Split type parser")
            }
            LogriaError::InvalidExampleRegex(msg) => {
                write!(fmt, "Invalid example: /{}/ has no captures", msg)
            }
            LogriaError::InvalidExampleSplit(msg, count) => write!(
                fmt,
                "Invalid example: {:?} matches for {:?} methods",
                msg, count
            ),
            LogriaError::CannotRead(path, why) => write!(fmt, "Couldn't open {:?}: {}", path, why),
            LogriaError::CannotWrite(path, why) => {
                write!(fmt, "Couldn't write {:?}: {}", path, why)
            }
            LogriaError::CannotRemove(path, why) => {
                write!(fmt, "Couldn't remove {:?}: {}", path, why)
            }
            LogriaError::CannotParseDate(msg) => {
                write!(fmt, "Invalid format description: {}", msg)
            }
            LogriaError::InvalidCommand(msg) => {
                write!(fmt, "Invalid poll command: {}", msg)
            }
            LogriaError::CannotParseMessage(msg) => {
                write!(fmt, "Unable to parse message: {}", msg)
            }
        }
    }
}
