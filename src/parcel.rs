use std::fs;
use std::io;
use std::io::Write;

use tempfile::Builder;

use crate::models::ResultWithDefaultError;
use crate::utilities;

fn io_err(context: &str, error: io::Error) -> Box<dyn std::error::Error + Send> {
    Box::new(io::Error::other(format!("{context}: {error}")))
}

pub trait Parcel {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: Vec<u8>, base: &Self) -> ResultWithDefaultError<Self>
    where
        Self: Sized;

    fn update_in_editor(&self) -> ResultWithDefaultError<Self>
    where
        Self: Sized,
    {
        let mut file = Builder::new()
            .suffix(".txt")
            .tempfile()
            .map_err(|e| io_err("create tempfile", e))?;
        file.write_all(&self.serialize())
            .map_err(|e| io_err("write tempfile", e))?;
        let path = file.into_temp_path();

        utilities::open_path_in_editor(&path)?;

        let contents = fs::read(&path).map_err(|e| io_err("read tempfile", e))?;
        path.close().map_err(|e| io_err("close tempfile", e))?;

        Self::deserialize(contents, self)
    }
}
