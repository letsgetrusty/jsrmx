use super::Writeable;
use rayon::prelude::*;
use serde_json::Value;
use std::{
    fs::{create_dir_all, OpenOptions},
    io::BufWriter,
    path::PathBuf,
};

#[derive(Clone, Debug)]
pub struct DirectoryOutput {
    pretty: bool,
    pub path: PathBuf,
}

impl DirectoryOutput {
    pub fn new(path: PathBuf, pretty: bool) -> Self {
        Self { pretty, path }
    }

    fn write_file(&self, filename: &str, content: Value) -> std::io::Result<()> {
        let mut path = self.path.clone();
        path.push(filename);
        log::info!("Writing file {}", path.display());
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;
        let mut writer = BufWriter::new(file);
        match self.pretty {
            true => serde_json::to_writer_pretty(&mut writer, &content)?,
            false => serde_json::to_writer(&mut writer, &content)?,
        }
        Ok(())
    }
}

impl Writeable for DirectoryOutput {
    fn append(&self, _content: Value) -> std::io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Cannot append to a directory output",
        ))
    }

    fn set_pretty(&mut self, pretty: bool) {
        self.pretty = pretty;
    }

    fn write_entries(&self, mut entries: Vec<(String, Value)>) -> std::io::Result<()> {
        if self.path != PathBuf::from(".") {
            //log::info!("Creating directory {}", self.path.display());
            create_dir_all(&self.path)?;
        }

        entries.par_drain(..).for_each(|(key, value)| {
            self.write_file(&format!("{key}.json"), value)
                .err()
                .map(|e| {
                    log::error!("Error writing to file: {e}");
                });
        });
        Ok(())
    }
}
