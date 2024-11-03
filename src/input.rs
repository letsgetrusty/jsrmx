pub mod file;

use serde_json::Value;
use std::{
    collections::HashMap,
    fs::File,
    io::{stdin, BufRead, BufReader, Read, Stdin},
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub trait JsonSource: Send + Sync {
    fn get_entries(&self, sort: bool) -> Vec<(String, Value)>;
}

pub trait JsonReader: Send + Sync {
    fn get_object(&self) -> std::io::Result<HashMap<String, Value>>;
    fn read_line(&self, buf: &mut String) -> Result<(), String>;
}

#[derive(Clone, Debug)]
pub struct InputDirectory(PathBuf);

pub struct InputFile {
    path: PathBuf,
    reader: Arc<Mutex<BufReader<File>>>,
}

struct InputStdin {
    reader: Arc<Mutex<BufReader<Stdin>>>,
}

impl JsonSource for InputDirectory {
    fn get_entries(&self, sort: bool) -> Vec<(String, Value)> {
        file::read_entries_from_directory(&self.0, sort)
            .expect("Error reading entries from directory")
    }
}

impl AsRef<PathBuf> for InputDirectory {
    fn as_ref(&self) -> &PathBuf {
        &self.0
    }
}

impl JsonSource for InputStdin {
    fn get_entries(&self, _sort: bool) -> Vec<(String, Value)> {
        let mut entries = Vec::new();
        let mut buf = String::new();
        while self.read_line(&mut buf).is_ok() {
            if buf.is_empty() {
                break;
            }
            match serde_json::from_str(&buf) {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    log::error!("Error parsing JSON: {}", e);
                }
            }
            buf.clear();
        }
        entries
    }
}

impl JsonReader for InputFile {
    fn get_object(&self) -> std::io::Result<HashMap<String, Value>> {
        file::read_hashmap(&self.path)
    }

    fn read_line(&self, buf: &mut String) -> Result<(), String> {
        let mut reader = self.reader.lock().map_err(|e| e.to_string())?;
        reader.read_line(buf).map_err(|e| e.to_string())?;
        Ok(())
    }
}

impl JsonReader for InputStdin {
    fn get_object(&self) -> std::io::Result<HashMap<String, Value>> {
        let mut buffer = String::new();
        stdin().read_to_string(&mut buffer)?;
        Ok(serde_json::from_str(&buffer)?)
    }

    fn read_line(&self, buf: &mut String) -> Result<(), String> {
        let mut reader = self.reader.lock().map_err(|e| e.to_string())?;
        reader.read_line(buf).map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct JsonSourceInput(pub Arc<dyn JsonSource>);

impl std::str::FromStr for JsonSourceInput {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "-" => Ok(JsonSourceInput(Arc::new(InputStdin {
                reader: Arc::new(Mutex::new(BufReader::new(stdin()))),
            }))),
            input => {
                let path = PathBuf::from(input);
                if path.is_dir() {
                    Ok(JsonSourceInput(Arc::new(InputDirectory(path))))
                } else {
                    Err(format!("Cannot read entries from file: {input}"))
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct JsonReaderInput(pub Arc<dyn JsonReader>);

impl std::str::FromStr for JsonReaderInput {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "-" => Ok(JsonReaderInput(Arc::new(InputStdin {
                reader: Arc::new(Mutex::new(BufReader::new(stdin()))),
            }))),
            input => {
                let path = PathBuf::from(input);
                if path.is_dir() {
                    Err(format!("Cannot read object from directory: {input}"))
                } else {
                    let file = File::open(&path).map_err(|e| e.to_string())?;
                    Ok(JsonReaderInput(Arc::new(InputFile {
                        reader: Arc::new(Mutex::new(BufReader::new(file))),
                        path,
                    })))
                }
            }
        }
    }
}

impl std::str::FromStr for InputDirectory {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Ok(InputDirectory(PathBuf::from(input)))
    }
}
