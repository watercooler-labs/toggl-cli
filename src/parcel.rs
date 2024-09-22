use std::fs::File;
use std::io::{Read, Seek, Write};

use tempfile::tempdir;

use crate::utilities;

pub trait Parcel {
    fn serialize(&self) -> String;
    fn deserialize(&self, data: &str) -> Self;

    fn launch_in_editor(&self) -> Result<Self, String>
    where
        Self: Sized,
    {
        let contents = self.serialize();

        let dir = tempdir().map_err(|e| e.to_string())?;
        let file_path = dir.path().join("toggl.txt");

        let mut file = File::create_new(file_path.clone()).map_err(|e| e.to_string())?;
        file.write_all(contents.as_bytes())
            .expect("Failed to write current time-entry to file");

        utilities::open_path_in_editor(file_path).map_err(|e| e.to_string())?;
        file.rewind().map_err(|e| e.to_string())?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| e.to_string())?;

        drop(file);
        dir.close().map_err(|e| e.to_string())?;

        Ok(self.deserialize(&contents))
    }
}
