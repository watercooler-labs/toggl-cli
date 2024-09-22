use std::fs::{self, File};
use std::io::Write;

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

        utilities::open_path_in_editor(&file_path).map_err(|e| e.to_string())?;
        drop(file);

        let contents = fs::read_to_string(file_path)
            .map_err(|e| e.to_string())?;

        dir.close().map_err(|e| e.to_string())?;

        Ok(self.deserialize(&contents))
    }
}
