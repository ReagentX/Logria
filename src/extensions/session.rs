use std::{
    collections::HashSet,
    error::Error,
    fs::{create_dir_all, read_dir, read_to_string, remove_file, write},
    path::Path,
};

use serde::{Deserialize, Serialize};

use crate::constants::{cli::excludes::SESSION_FILE_EXCLUDES, directories::sessions};

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Debug)]
pub enum SessionType {
    File,
    Command,
    Mixed,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Session {
    pub commands: Vec<String>,
    pub stream_type: SessionType, // Cannot use `type` for the name as it is reserved
}

impl Session {
    /// Ensure the proper paths exist
    pub fn verify_path() {
        let tape_path = sessions();
        if !Path::new(&tape_path).exists() {
            create_dir_all(tape_path).unwrap();
        }
    }

    /// Create a Session struct
    pub fn new(commands: &Vec<String>, stream_type: SessionType) -> Session {
        Session::verify_path();
        Session {
            commands: commands.to_owned(),
            stream_type,
        }
    }

    /// Create session file from a Session struct
    pub fn save(self, file_name: &str) {
        let session_json = serde_json::to_string_pretty(&self).unwrap();
        let path = format!("{}/{}", sessions(), file_name);
        match write(&path, session_json) {
            Ok(_) => {}
            Err(why) => panic!("Couldn't write {:?}: {}", path, Error::to_string(&why)),
        }
    }

    /// Create Session struct from a session file
    pub fn load(file_name: &str) -> Result<Session, serde_json::error::Error> {
        // Read file
        let session_json = match read_to_string(file_name) {
            Ok(json) => json,
            Err(why) => panic!("Couldn't open {:?}: {}", file_name, Error::to_string(&why)),
        };
        let session = serde_json::from_str(&session_json);
        match session {
            Ok(s) => Ok(s),
            Err(e) => Err(e),
        }
    }

    /// Delete the path for a fully qualified session filename
    pub fn del(items: &Vec<usize>) {
        // Iterate through each `i` in `items` and remove the item at list index `i`
        let files = Session::list();
        for i in items {
            if i >= &files.len() {
                break;
            }
            let file_name = &files[*i];
            match remove_file(file_name) {
                Ok(_) => {}
                Err(why) => panic!(
                    "Couldn't remove {:?}: {}",
                    file_name,
                    Error::to_string(&why)
                ),
            }
        }
    }

    /// Get a list of all available session configurations
    pub fn list() -> Vec<String> {
        // Files to exclude from the session list
        let mut excluded = HashSet::new();
        for &item in &SESSION_FILE_EXCLUDES {
            excluded.insert(format!("{}/{}", sessions(), item));
        }

        let mut sessions: Vec<String> = read_dir(sessions())
            .unwrap()
            .map(|session| String::from(session.unwrap().path().to_str().unwrap()))
            .filter(|item| !excluded.contains(item))
            .collect();
        sessions.sort();
        sessions
    }
}

#[cfg(test)]
mod tests {
    use super::{Session, SessionType};
    use crate::constants::directories::sessions;

    #[test]
    fn test_list() {
        let list = Session::list();
        assert!(list
            .iter()
            .any(|i| i == &format!("{}/{}", sessions(), "ls -la")))
    }

    #[test]
    fn serialize_session() {
        let session = Session::new(&vec![String::from("ls -la")], SessionType::Command);
        session.save("ls -la")
    }

    #[test]
    fn deserialize_session() {
        let read_session = Session::load(&format!("{}/{}", sessions(), "ls -la copy")).unwrap();
        let expected_session = Session {
            commands: vec![String::from("ls -la")],
            stream_type: SessionType::Command,
        };
        assert_eq!(read_session.commands, expected_session.commands);
        assert_eq!(read_session.stream_type, expected_session.stream_type);
    }

    #[test]
    fn delete_session() {
        let session = Session::new(&vec![String::from("ls -la")], SessionType::Command);
        session.save("zzzfake_file_name");
        Session::del(&vec![Session::list().len() - 1]);
    }
}
