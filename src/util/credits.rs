use crate::constants::app::LOGRIA;

// TODO: abstract this, do something cooler
pub fn gen() -> Vec<String> {
    LOGRIA.into_iter().map(|s| s.to_owned()).collect()
}
