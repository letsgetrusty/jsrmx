mod file;

use file::{read_entries_from_directory, read_hashmap};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::{fmt, str};

#[derive(Clone, Debug)]
pub enum Input {
    Stdin {
        reader: Arc<Mutex<BufReader<io::Stdin>>>,
    },
    File {
        path: PathBuf,
        reader: Arc<Mutex<BufReader<File>>>,
    },
    Directory(PathBuf),
}

impl Input {
    pub fn read_line(&self, buf: &mut String) -> Result<(), String> {
        match self {
            Input::Stdin { reader } => {
                let mut reader = reader.lock().map_err(|e| e.to_string())?;
                reader.read_line(buf).map_err(|e| e.to_string())?;
                Ok(())
            }
            Input::File { reader, .. } => {
                let mut reader = reader.lock().map_err(|e| e.to_string())?;
                reader.read_line(buf).map_err(|e| e.to_string())?;
                Ok(())
            }
            Input::Directory(path) => Err(format!(
                "Cannot read_line from directory {}",
                path.display()
            )),
        }
    }

    pub fn get_entries(&self, sort: bool) -> Vec<(String, Value)> {
        match self {
            Input::Stdin { .. } => {
                let mut entries = Vec::new();
                let mut buf = String::new();
                while let Ok(_) = self.read_line(&mut buf) {
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
            Input::File { path, .. } => {
                panic!("Cannot read entries from file: {path:?}");
            }
            Input::Directory(dir) => match read_entries_from_directory(dir, sort) {
                Ok(entries) => entries,
                Err(e) => {
                    panic!("Error reading entries from directory: {}", e);
                }
            },
        }
    }

    pub fn get_object(&self) -> std::io::Result<HashMap<String, Value>> {
        match self {
            Input::Stdin { .. } => {
                let mut buffer = String::new();
                std::io::stdin().read_to_string(&mut buffer)?;
                Ok(serde_json::from_str(&buffer)?)
            }
            Input::File { path, .. } => Ok(read_hashmap(path)?),
            Input::Directory(input) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Cannot split a directory: {input:?}"),
            )),
        }
    }
}

impl fmt::Display for Input {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Input::Stdin { .. } => write!(f, "-"),
            Input::File { path, .. } => write!(f, "{}", path.display()),
            Input::Directory(path) => write!(f, "{}", path.display()),
        }
    }
}

impl str::FromStr for Input {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "-" => Ok(Input::Stdin {
                reader: Arc::new(Mutex::new(BufReader::new(io::stdin()))),
            }),
            input => {
                let path = PathBuf::from(input);
                match path.is_dir() {
                    true => Ok(Input::Directory(path)),
                    false => Ok(Input::File {
                        reader: Arc::new(Mutex::new(BufReader::new(
                            File::open(&path).map_err(|e| e.to_string())?,
                        ))),
                        path,
                    }),
                }
            }
        }
    }
}

impl AsRef<Path> for Input {
    fn as_ref(&self) -> &Path {
        match self {
            Input::Stdin { .. } => Path::new("-"),
            Input::File { path, .. } => path.as_path(),
            Input::Directory(path) => path.as_path(),
        }
    }
}

impl AsRef<PathBuf> for Input {
    fn as_ref(&self) -> &PathBuf {
        match self {
            Input::Stdin { .. } => panic!("Cannot convert stdin to PathBuf"),
            Input::File { path, .. } => path,
            Input::Directory(path) => path,
        }
    }
}
