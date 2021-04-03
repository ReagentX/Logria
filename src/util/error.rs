use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum LogriaError {
    InvalidRegex(String),
    InvalidSelection(String),
    WrongParserType,
    InvalidExampleRegex(String),
    InvalidExampleSplit(usize, usize),
    CannotRead(String, String),
    CannotWrite(String, String),
}

impl Display for LogriaError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        match self {
            LogriaError::InvalidRegex(msg) => write!(fmt, "{}", msg),
            LogriaError::InvalidSelection(msg) => write!(fmt, "{}", msg),
            LogriaError::WrongParserType => {
                write!(fmt, "Cannot construct regex for a Split type parser")
            },
            LogriaError::InvalidExampleRegex(msg) => {
                write!(fmt, "Invalid example: /{}/ has no captures", msg)
            },
            LogriaError::InvalidExampleSplit(msg, count) => write!(
                fmt,
                "Invalid example: {:?} matches for {:?} methods",
                msg, count
            ),
            LogriaError::CannotRead(path, why) => {
                write!(fmt, "Couldn't open {:?}: {}", path, why)
            },
            LogriaError::CannotWrite(path, why) => {
                write!(fmt, "Couldn't write {:?}: {}", path, why)
            },
        }
    }
}