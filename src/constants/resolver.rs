use dirs::home_dir;
use std::env;

pub fn get_home_dir() -> String {
    match env::var("LOGRIA_DISABLE_USER_HOME") {
        Ok(val) => val,
        Err(_) => home_dir()
            .to_owned()
            .expect("Unable to start application: home directory not resolved!")
            .to_str()
            .expect("Home directory path is badly malformed!")
            .to_string(),
    }
}

pub fn get_env_var_or_default(var_name: &str, default: &'static str) -> String {
    match env::var(var_name) {
        Ok(val) => val,
        Err(_) => default.to_owned(),
    }
}
