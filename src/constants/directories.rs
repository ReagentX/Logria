use std::env;
use crate::constants::{
    app::NAME,
    resolver::{get_env_var_or_default, get_home_dir},
};

// Paths
pub fn home() -> String {
    get_home_dir()
}

pub fn app_root() -> String {
    let mut root = home();
    root.push('/');
    root.push_str(&get_env_var_or_default("LOGRIA_ROOT", NAME));
    root
}

pub fn patterns() -> String {
    let mut root = app_root();
    root.push_str("/parsers");
    root
}

pub fn sessions() -> String {
    let mut root = app_root();
    root.push_str("/sessions");
    root
}

pub fn history() -> String {
    let mut root = app_root();
    root.push_str("/history");
    root
}

pub fn history_tape() -> String {
    let mut root = app_root();
    root.push_str("/history/tape");
    root
}

pub fn print_paths() {
    let mut result = String::new();
    result.push_str("Environment variables:\n");
    match env::var("LOGRIA_USER_HOME") {
        Ok(home) => {
            result.push_str(&format!("LOGRIA_USER_HOME: {}\n", home));
        },
        Err(_) => {
            result.push_str("LOGRIA_USER_HOME: Not Set\n");
        },
    };
    match env::var("LOGRIA_ROOT") {
        Ok(root) => {
            result.push_str(&format!("LOGRIA_ROOT: {}\n", root));
        },
        Err(_) => {
            result.push_str("LOGRIA_ROOT:      Not Set\n");
        },
    };

    result.push_str("\nExpanded paths:\n");
    result.push_str(&format!("Config root: {}\n", home()));
    result.push_str(&format!("Logria root: {}\n", app_root()));
    result.push_str(&format!("Patterns:    {}\n", patterns()));
    result.push_str(&format!("Sessions:    {}\n", sessions()));
    result.push_str(&format!("History:     {}", history()));
    println!("{}", result);
}

#[cfg(test)]
mod tests {
    use crate::constants::directories;
    use dirs::config_dir;

    #[test]
    fn test_app_root() {
        let t = directories::app_root();
        let mut root = config_dir().unwrap().to_str().unwrap().to_string();
        root.push_str("/Logria");
        assert_eq!(t, root)
    }

    #[test]
    fn test_patterns() {
        let t = directories::patterns();
        let mut root = config_dir().unwrap().to_str().unwrap().to_string();
        root.push_str("/Logria/parsers");
        assert_eq!(t, root)
    }

    #[test]
    fn test_sessions() {
        let t = directories::sessions();
        let mut root = config_dir().expect("").to_str().expect("").to_string();
        root.push_str("/Logria/sessions");
        assert_eq!(t, root)
    }

    #[test]
    fn test_history() {
        let t = directories::history();
        let mut root = config_dir().expect("").to_str().expect("").to_string();
        root.push_str("/Logria/history");
        assert_eq!(t, root)
    }

    #[test]
    fn test_history_tape() {
        let t = directories::history_tape();
        let mut root = config_dir().expect("").to_str().expect("").to_string();
        root.push_str("/Logria/history/tape");
        assert_eq!(t, root)
    }

    #[test]
    fn test_print_paths() {
        // Ensure no weird crashes here
        directories::print_paths();
    }
}
