use crate::constants::resolver::{get_env_var_or_default, get_home_dir};

// Paths
fn home() -> String {
    println!("{}", get_home_dir());
    get_home_dir()
}

fn app_root() -> String {
    let mut root = home();
    root.push_str("/");
    root.push_str(&get_env_var_or_default("LOGRIA_ROOT", ".logria"));
    return root;
}

fn patterns() -> String {
    let mut root = app_root();
    root.push_str("/patterns");
    root
}

fn sessions() -> String {
    let mut root = app_root();
    root.push_str("/sessions");
    root
}

fn history() -> String {
    let mut root = app_root();
    root.push_str("/history");
    root
}

fn history_tape() -> String {
    let mut root = app_root();
    root.push_str("/history/tape");
    root
}


#[cfg(test)]
mod tests {
    use crate::constants::directories;
    use dirs::home_dir;

    #[test]
    fn test_app_root() {
        let t = directories::app_root();
        let mut root = home_dir().expect("").to_str().expect("").to_string();
        root.push_str("/.logria");
        assert_eq!(t, root)
    }

    #[test]
    fn test_patterns() {
        let t = directories::patterns();
        let mut root = home_dir().expect("").to_str().expect("").to_string();
        root.push_str("/.logria/patterns");
        assert_eq!(t, root)
    }

    #[test]
    fn test_sessions() {
        let t = directories::sessions();
        let mut root = home_dir().expect("").to_str().expect("").to_string();
        root.push_str("/.logria/sessions");
        assert_eq!(t, root)
    }

    #[test]
    fn test_history() {
        let t = directories::history();
        let mut root = home_dir().expect("").to_str().expect("").to_string();
        root.push_str("/.logria/history");
        assert_eq!(t, root)
    }

    #[test]
    fn test_history_tape() {
        let t = directories::history_tape();
        let mut root = home_dir().expect("").to_str().expect("").to_string();
        root.push_str("/.logria/history/tape");
        assert_eq!(t, root)
    }
}
