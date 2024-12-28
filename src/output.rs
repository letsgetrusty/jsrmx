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
    path::PathBuf,
    sync::{Arc, RwLock},
};

pub trait Appendable: Writeable {
    fn append(&self, content: Value) -> std::io::Result<()>;
}

pub trait Writeable: Send + Sync {
    fn set_pretty(&mut self, pretty: bool);
    fn write_entries(&self, entries: Vec<(String, Value)>) -> std::io::Result<()>;
}

#[derive(Clone)]
pub struct JsonAppendableOutput(pub Arc<RwLock<dyn Appendable>>);

impl std::str::FromStr for JsonAppendableOutput {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "-" => Ok(JsonAppendableOutput(Arc::new(RwLock::new(
                StreamOutput::new(false),
            )))),
            s => {
                let path = PathBuf::from(s);
                if path.is_dir() | path.extension().is_none() {
                    Err(eyre!("Cannot append to a directory output: {s}"))
                } else if path.is_file() {
                    Ok(JsonAppendableOutput(Arc::new(RwLock::new(
                        FileOutput::new(path, false),
                    ))))
                } else {
                    log::info!("Creating file: {}", &path.display());
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

#[derive(Clone)]
pub struct JsonWritableOutput(pub Arc<RwLock<dyn Writeable>>);

impl std::str::FromStr for JsonWritableOutput {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "-" => Ok(JsonWritableOutput(Arc::new(RwLock::new(
                StreamOutput::new(false),
            )))),
            s => {
                let path = PathBuf::from(s);
                if path.is_dir() | path.extension().is_none() {
                    Ok(JsonWritableOutput(Arc::new(RwLock::new(
                        DirectoryOutput::new(path, false),
                    ))))
                } else if path.is_file() {
                    Ok(JsonWritableOutput(Arc::new(RwLock::new(FileOutput::new(
                        path, false,
                    )))))
                } else {
                    log::info!("Creating file: {}", &path.display());
                    Ok(JsonWritableOutput(Arc::new(RwLock::new(FileOutput::new(
                        path, false,
                    )))))
                }
            }
        }
    }
}

impl Deref for JsonWritableOutput {
    type Target = Arc<RwLock<dyn Writeable>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
