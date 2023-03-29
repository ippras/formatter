use anyhow::{bail, Result};
use egui::DroppedFile;
use std::fs::read_to_string;

/// Extension methods for [`DroppedFile`]
pub trait DroppedFileExt {
    fn content(&self) -> Result<String>;
}

impl DroppedFileExt for DroppedFile {
    fn content(&self) -> Result<String> {
        Ok(match &self.bytes {
            Some(bytes) => String::from_utf8(bytes.to_vec())?,
            None => match &self.path {
                Some(path) => read_to_string(&path)?,
                None => bail!("Dropped file hasn't bytes or path"),
            },
        })
    }
}
