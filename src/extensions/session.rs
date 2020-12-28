use std::{fs::read_to_string, error::Error};

use serde::{Serialize, Deserialize};

use crate::constants::directories::sessions;

#[derive(Serialize, Deserialize, Debug)]
pub struct Session {
    commands: Vec<String>,
    genre: String
}

impl Session {
    fn serialize(self) {
        serde_json::to_string(&self).unwrap();
    }

    fn deserialize(self, file_name: String) -> Session {
        // Read file
        let config_json = match read_to_string(format!("{}/{}", sessions(), file_name)) {
            Ok(json) => json,
            Err(why) => panic!(
                "Couldn't open {:?}: {}",
                sessions(),
                Error::to_string(&why)
            ),
        };
        let session: Session = serde_json::from_str(&config_json).unwrap();
        session
    }
}
