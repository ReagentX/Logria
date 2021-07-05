use crate::util::error::LogriaError;
use std::result::Result;

pub type Del = Option<fn(&[usize]) -> Result<(), LogriaError>>;
