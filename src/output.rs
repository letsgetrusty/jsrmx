mod file;

use file::write_object;
use rayon::prelude::*;
use serde_json::Value;
use std::{
    fmt,
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
    str,
    sync::{Arc, Mutex},
};

#[derive(Clone, Debug)]
pub enum Output {
    Directory {
        path: PathBuf,
        pretty: bool,
    },
    File {
        path: PathBuf,
        pretty: bool,
        writer: Arc<Mutex<BufWriter<File>>>,
    },
    Stdout {
        pretty: bool,
    },
}

impl Output {
    pub fn append(&self, value: Value) -> io::Result<()> {
        match self {
            Self::Directory { .. } => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Cannot append to a directory",
                ))
            }
            Self::File { pretty, writer, .. } => {
                let mut guard = writer.lock().expect("Failed to get writer lock");
                match pretty {
                    true => serde_json::to_writer_pretty(&mut *guard, &value)?,
                    false => serde_json::to_writer(&mut *guard, &value)?,
                }
                writeln!(&mut *guard)?;
            }
            Self::Stdout { pretty } => {
                match pretty {
                    true => serde_json::to_writer_pretty(io::stdout(), &value)?,
                    false => serde_json::to_writer(io::stdout(), &value)?,
                }
                println!();
            }
        }
        Ok(())
    }

    pub fn set_pretty(&mut self) {
        match self {
            Self::Directory { pretty, .. } => *pretty = true,
            Self::File { pretty, .. } => *pretty = true,
            Self::Stdout { pretty, .. } => *pretty = true,
        }
    }

    pub fn write(&self, path: &PathBuf, value: Value) -> io::Result<()> {
        match self {
            Self::Directory { pretty, .. } => {
                write_object(path, value, *pretty)?;
            }
            Self::File { pretty, .. } => {
                write_object(path, value, *pretty)?;
            }
            Self::Stdout { pretty } => match pretty {
                true => serde_json::to_writer_pretty(io::stdout(), &value)?,
                false => serde_json::to_writer(io::stdout(), &value)?,
            },
        }
        Ok(())
    }

    pub fn write_entries(&self, mut entries: Vec<(String, Value)>) -> io::Result<()> {
        match self {
            Output::Directory { path, .. } => {
                let dir = path.to_str().expect("Invalid directory");
                // Don't create the directory if it is the present working directory
                if dir != "." {
                    log::info!("Creating directory {}", dir);
                    fs::create_dir_all(dir)?;
                }
                entries.par_drain(..).for_each(|(key, value)| {
                    let filename = Path::new(dir).join(format!("{key}.json"));
                    match self.write(&filename, value.clone()) {
                        Ok(_) => (),
                        Err(e) => log::error!("Error writing to file: {e}"),
                    }
                });
            }
            Output::File { pretty, .. } => {
                if *pretty {
                    log::warn!("Pretty printing not recommended for writing entries to a file!");
                }
                entries.par_drain(..).for_each(|(key, value)| {
                    match self.append(serde_json::json!({key: value})) {
                        Ok(_) => (),
                        Err(e) => log::error!("Error writing to file: {e}"),
                    }
                })
            }
            Output::Stdout { .. } => {
                for (key, value) in entries {
                    self.append(serde_json::json!({key: value}))?;
                }
            }
        }
        Ok(())
    }
}

impl AsRef<Path> for Output {
    fn as_ref(&self) -> &Path {
        match self {
            Output::Directory { path, .. } => path.as_path(),
            Output::File { path, .. } => path.as_path(),
            Output::Stdout { .. } => Path::new("-"),
        }
    }
}

impl fmt::Display for Output {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Output::Directory { path, .. } => write!(f, "{}", path.display()),
            Output::File { path, .. } => write!(f, "{}", path.display()),
            Output::Stdout { .. } => write!(f, "-"),
        }
    }
}

impl str::FromStr for Output {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "-" {
            return Ok(Output::Stdout { pretty: false });
        }

        let path = PathBuf::from(s);
        if path.is_dir() {
            Ok(Output::Directory {
                path,
                pretty: false,
            })
        } else if path.extension().is_none() {
            Ok(Output::Directory {
                path,
                pretty: false,
            })
        } else if path.is_file() {
            Ok(Output::File {
                writer: fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .append(true)
                    .open(&path)
                    .map(|f| Arc::new(Mutex::new(BufWriter::new(f))))
                    .map_err(|e| e.to_string())?,
                path,
                pretty: false,
            })
        } else {
            log::info!("Creating file: {}", &path.display());
            Ok(Output::File {
                writer: fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .append(true)
                    .open(&path)
                    .map(|f| Arc::new(Mutex::new(BufWriter::new(f))))
                    .map_err(|e| e.to_string())?,
                path,
                pretty: false,
            })
        }
    }
}
