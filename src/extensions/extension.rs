use crate::util::error::LogriaError;
use std::result::Result;

pub trait ExtensionMethods {
    fn new() -> Self;
    fn save() -> Result<(), LogriaError>;
    fn load<T>() -> Result<T, serde_json::error::Error>;
    fn del() -> Result<(), LogriaError>;
    fn list() -> Vec<String>;
}
