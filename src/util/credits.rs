use crate::constants::app::LOGRIA;

pub fn gen_credits() -> Vec<String> {
    LOGRIA
        .into_iter()
        .map(std::borrow::ToOwned::to_owned)
        .collect()
}
