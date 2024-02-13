use std::fs::OpenOptions;
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

        // TODO: Replace with `File::create_new` when it's stable
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(file_path.clone())
            .map_err(|e| e.to_string())?;
        write!(file, "{}", contents).unwrap();

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
