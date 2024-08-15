mod file;

use file::{read_entries_from_directory, read_hashmap};
use serde_json::Value;
use std::{
    collections::HashMap,
    fmt,
    fs::File,
    io::{self, BufRead, BufReader, Read},
    path::{Path, PathBuf},
    str,
    sync::{Arc, Mutex},
};

#[derive(Clone, Debug)]
pub enum Input {
    Directory(PathBuf),
    File {
        path: PathBuf,
        reader: Arc<Mutex<BufReader<File>>>,
    },
    Stdin {
        reader: Arc<Mutex<BufReader<io::Stdin>>>,
    },
}

impl Input {
    pub fn get_entries(&self, sort: bool) -> Vec<(String, Value)> {
        match self {
            Input::Directory(dir) => match read_entries_from_directory(dir, sort) {
                Ok(entries) => entries,
                Err(e) => {
                    panic!("Error reading entries from directory: {}", e);
                }
            },
            Input::File { path, .. } => {
                panic!("Cannot read entries from file: {path:?}");
            }
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
        }
    }

    pub fn get_object(&self) -> io::Result<HashMap<String, Value>> {
        match self {
            Input::Directory(input) => Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Cannot split a directory: {input:?}"),
            )),
            Input::File { path, .. } => Ok(read_hashmap(path)?),
            Input::Stdin { .. } => {
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)?;
                Ok(serde_json::from_str(&buffer)?)
            }
        }
    }

    pub fn read_line(&self, buf: &mut String) -> Result<(), String> {
        match self {
            Input::Directory(path) => Err(format!(
                "Cannot read_line from directory {}",
                path.display()
            )),
            Input::File { reader, .. } => {
                let mut reader = reader.lock().map_err(|e| e.to_string())?;
                reader.read_line(buf).map_err(|e| e.to_string())?;
                Ok(())
            }
            Input::Stdin { reader } => {
                let mut reader = reader.lock().map_err(|e| e.to_string())?;
                reader.read_line(buf).map_err(|e| e.to_string())?;
                Ok(())
            }
        }
    }
}

impl AsRef<Path> for Input {
    fn as_ref(&self) -> &Path {
        match self {
            Input::Directory(path) => path.as_path(),
            Input::File { path, .. } => path.as_path(),
            Input::Stdin { .. } => Path::new("-"),
        }
    }
}

impl AsRef<PathBuf> for Input {
    fn as_ref(&self) -> &PathBuf {
        match self {
            Input::Directory(path) => path,
            Input::File { path, .. } => path,
            Input::Stdin { .. } => panic!("Cannot convert stdin to PathBuf"),
        }
    }
}

impl fmt::Display for Input {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Input::Directory(path) => write!(f, "{}", path.display()),
            Input::File { path, .. } => write!(f, "{}", path.display()),
            Input::Stdin { .. } => write!(f, "-"),
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
                if path.is_dir() {
                    Ok(Input::Directory(path))
                } else {
                    let file = File::open(&path).map_err(|e| e.to_string())?;
                    Ok(Input::File {
                        reader: Arc::new(Mutex::new(BufReader::new(file))),
                        path,
                    })
                }
            }
        }
    }
}
