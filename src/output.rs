pub mod directory;
pub mod file;
pub mod stream;

use serde::Serialize;
use serde_json::Value;
use std::path::PathBuf;

pub trait FileAndStdOut: AllOutputs {
    fn append<T: Serialize>(&self, content: T) -> std::io::Result<()>;
    fn write<T: Serialize>(&self, content: T) -> std::io::Result<()>;
}

pub trait AllOutputs {
    fn set_pretty(&mut self, pretty: bool);
    fn write_entries(&self, entries: Vec<(String, Value)>) -> std::io::Result<()>;
}

#[derive(Clone, Debug)]
pub enum Output {
    Directory(PathBuf),
    File(PathBuf),
    Stdout,
}

impl std::str::FromStr for Output {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "-" {
            return Ok(Output::Stdout);
        }

        let path = PathBuf::from(s);
        if path.is_dir() || path.extension().is_none() {
            Ok(Output::Directory(path))
        } else if path.is_file() {
            Ok(Output::File(path))
        } else {
            log::info!("Creating file: {}", &path.display());
            Ok(Output::File(path))
        }
    }
}
