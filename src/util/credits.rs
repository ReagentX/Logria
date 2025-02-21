use crate::constants::app::LOGRIA;

pub fn gen_credits() -> Vec<String> {
    LOGRIA.into_iter().map(|s| s.to_owned()).collect()
}
