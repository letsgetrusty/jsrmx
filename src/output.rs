mod directory;
mod file;
mod stream;
use directory::DirectoryOutput;
use eyre::{eyre, Report, Result};
use file::FileOutput;
use stream::StreamOutput;

use serde_json::Value;
use std::{
    ops::Deref,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

pub trait Appendable: Writeable {
    fn append(&self, content: Value) -> std::io::Result<()>;
}

pub trait Writeable: Send + Sync {
    fn set_pretty(&mut self, pretty: bool);
    fn write_entries(&self, entries: Vec<(String, Value)>) -> std::io::Result<()>;
}

#[derive(Clone, Debug)]
pub enum Output {
    Directory(DirectoryOutput),
    File(FileOutput),
    Stdout(StreamOutput),
}

impl Output {
    pub fn append(&self, content: Value) -> std::io::Result<()> {
        match self {
            Self::Directory(_) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Cannot append to a directory output",
            )),
            Self::File(output) => output.append(content),
            Self::Stdout(output) => output.append(content),
        }
    }

    pub fn set_pretty(&mut self) {
        match self {
            Self::Directory(output) => output.set_pretty(true),
            Self::File(output) => output.set_pretty(true),
            Self::Stdout(output) => output.set_pretty(true),
        }
    }

    pub fn write_entries(&self, entries: Vec<(String, Value)>) -> std::io::Result<()> {
        match self {
            Self::Directory(output) => output.write_entries(entries),
            Self::File(output) => output.write_entries(entries),
            Self::Stdout(output) => output.write_entries(entries),
        }
    }
}

impl AsRef<Path> for Output {
    fn as_ref(&self) -> &Path {
        match self {
            Output::Directory(output) => output.path.as_path(),
            Output::File(output) => output.path.as_path(),
            Output::Stdout(_) => Path::new("-"),
        }
    }
}

impl std::fmt::Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Output::Directory(output) => write!(f, "{}", output.path.display()),
            Output::File(output) => write!(f, "{}", output.path.display()),
            Output::Stdout(_) => write!(f, "-"),
        }
    }
}

impl std::str::FromStr for Output {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "-" {
            return Ok(Output::Stdout(StreamOutput::new(false)));
        }

        let path = PathBuf::from(s);
        if path.is_dir() {
            Ok(Output::Directory(DirectoryOutput::new(path, false)))
        } else if path.extension().is_none() {
            Ok(Output::Directory(DirectoryOutput::new(path, false)))
        } else if path.is_file() {
            Ok(Output::File(FileOutput::new(path, false)))
        } else {
            log::info!("Creating file: {}", &path.display());
            Ok(Output::File(FileOutput::new(path, false)))
        }
    }
}

#[derive(Clone)]
pub struct JsonAppendableOutput(pub Arc<RwLock<dyn Appendable>>);

impl std::str::FromStr for JsonAppendableOutput {
    type Err = Report;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "-" => Ok(JsonAppendableOutput(Arc::new(RwLock::new(
                StreamOutput::new(false),
            )))),
            input => {
                let path = PathBuf::from(input);
                if path.is_dir() {
                    Err(eyre!("Cannot append to a directory output: {input}"))
                } else {
                    Ok(JsonAppendableOutput(Arc::new(RwLock::new(
                        FileOutput::new(path, false),
                    ))))
                }
            }
        }
    }
}

impl Deref for JsonAppendableOutput {
    type Target = Arc<RwLock<dyn Appendable>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
