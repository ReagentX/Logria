use std::{
    collections::HashSet,
    error::Error,
    fs::{create_dir_all, read_dir, read_to_string, remove_file, write},
    path::Path,
    result::Result,
};

use serde::{Deserialize, Serialize};

use crate::{
    constants::{cli::excludes::SESSION_FILE_EXCLUDES, directories::sessions},
    extensions::extension::ExtensionMethods,
    util::error::LogriaError,
};

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

impl ExtensionMethods for Session {
    /// Ensure the proper paths exist
    fn verify_path() {
        let tape_path = sessions();
        if !Path::new(&tape_path).exists() {
            create_dir_all(tape_path).unwrap();
        }
    }

    /// Create session file from a Session struct
    fn save(self, file_name: &str) -> Result<(), LogriaError> {
        let session_json = serde_json::to_string_pretty(&self).unwrap();
        let path = format!("{}/{}", sessions(), file_name);
        match write(&path, session_json) {
            Ok(_) => Ok(()),
            Err(why) => Err(LogriaError::CannotWrite(path, <dyn Error>::to_string(&why))),
        }
    }

    /// Delete the path for a fully qualified session filename
    fn del(items: &[usize]) -> Result<(), LogriaError> {
        // Iterate through each `i` in `items` and remove the item at list index `i`
        let files = Session::list();
        for i in items {
            if i >= &files.len() {
                break;
            }
            let file_name = &files[*i];
            match remove_file(file_name) {
                Ok(_) => {}
                // TODO: Make this return a LogriaError
                Err(why) => {
                    return Err(LogriaError::CannotRemove(
                        file_name.to_owned(),
                        <dyn Error>::to_string(&why),
                    ))
                }
            }
        }
        Ok(())
    }

    /// Get a list of all available session configurations
    fn list() -> Vec<String> {
        Session::verify_path();
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

impl Session {
    /// Create a Session struct
    pub fn new(commands: &[String], session_type: SessionType) -> Session {
        Session::verify_path();
        Session {
            commands: commands.to_owned(),
            stream_type: session_type,
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
}

#[cfg(test)]
mod tests {
    use crate::{
        constants::directories::sessions,
        extensions::{
            extension::ExtensionMethods,
            session::{Session, SessionType},
        },
    };
    use std::path::Path;

    #[test]
    fn test_list() {
        let list = Session::list();
        assert!(list
            .iter()
            .any(|i| i == &format!("{}/{}", sessions(), "ls -la")))
    }

    #[test]
    fn serialize_session() {
        let session = Session::new(&[String::from("ls -la")], SessionType::Command);
        session.save("ls -la").unwrap();

        assert!(Path::new(&format!("{}/{}", sessions(), "ls -la")).exists());
    }

    #[test]
    fn deserialize_session() {
        let session = Session::new(&[String::from("ls -la")], SessionType::Command);
        session.save("ls -la copy").unwrap();
        assert!(Path::new(&format!("{}/{}", sessions(), "ls -la copy")).exists());

        let file_name = format!("{}/{}", sessions(), "ls -la copy");
        let read_session: Session = Session::load(&file_name).unwrap();
        let expected_session = Session {
            commands: vec![String::from("ls -la")],
            stream_type: SessionType::Command,
        };
        assert_eq!(read_session.commands, expected_session.commands);
        assert_eq!(read_session.stream_type, expected_session.stream_type);
    }

    #[test]
    fn delete_session() {
        let session = Session::new(&[String::from("ls -la")], SessionType::Command);
        session.save("zzzfake_file_name").unwrap();
        Session::del(&[Session::list().len() - 1]).unwrap();
    }
}
