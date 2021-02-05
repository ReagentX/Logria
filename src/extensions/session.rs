use std::{
    error::Error,
    fs::{read_dir, read_to_string, write},
};

use serde::{Deserialize, Serialize};

use crate::constants::directories::sessions;

#[derive(Serialize, Deserialize, Debug)]
pub struct Session {
    commands: Vec<String>,
    stream_type: String, // Cannot use `type` for the name as it is reserved
}

impl Session {
    /// Create a Session struct
    fn new(commands: Vec<String>, stream_type: String) -> Session {
        Session {
            commands: commands,
            stream_type: stream_type
        }
    }

    /// Create session file from a Session struct
    fn save(self) {
        let session_json = serde_json::to_string_pretty(&self).unwrap();
        let file_name = self.commands.join(" ");
        let path = format!("{}/{}", sessions(), file_name);

        match write(format!("{}/{}", sessions(), file_name), session_json) {
            Ok(_) => {}
            Err(why) => panic!("Couldn't write {:?}: {}", path, Error::to_string(&why)),
        }
    }

    /// Create Session struct from a session file
    fn load(file_name: &str) -> Session {
        // Read file
        let path = format!("{}/{}", sessions(), file_name);
        let session_json = match read_to_string(path) {
            Ok(json) => json,
            Err(why) => panic!("Couldn't open {:?}: {}", sessions(), Error::to_string(&why)),
        };
        let session: Session = serde_json::from_str(&session_json).unwrap();
        session
    }

    fn list() -> Vec<String> {
        let mut sessions: Vec<String> = read_dir(sessions())
            .unwrap()
            .map(|session| String::from(session.unwrap().path().to_str().unwrap()))
            .collect();
        sessions.sort();
        sessions
    }
}

#[cfg(test)]
mod tests {
    use super::Session;
    use crate::constants::directories::sessions;

    #[test]
    fn test_list() {
        let list = Session::list();
        assert!(list.iter().any(|i| i==&format!("{}/{}", sessions(), "ls -la")))
    }

    #[test]
    fn serialize_session() {
        let session = Session::new(vec![String::from("ls -la")], String::from("command"));
        session.save()
    }

    #[test]
    fn deserialize_session() {
        let read_session = Session::load("ls -la copy");
        let expected_session = Session {
            commands: vec![String::from("ls -la")],
            stream_type: String::from("command"),
        };
        assert_eq!(read_session.commands, expected_session.commands);
        assert_eq!(read_session.stream_type, expected_session.stream_type);
    }
}
