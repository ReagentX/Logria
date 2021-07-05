use crate::util::error::LogriaError;
use std::result::Result;

pub trait ExtensionMethods {
    fn verify_path();
    fn save(self, file_name: &str) -> Result<(), LogriaError>;
    fn del(items: &[usize]) -> Result<(), LogriaError>;
    fn list_full() -> Vec<String>;
    fn list_clean() -> Vec<String>;
}
