use super::{file::read_object, JsonSource};
use eyre::Result;
use serde_json::Value;
use std::path::PathBuf;

#[derive(Clone)]
pub struct InputDirectory {
    path: PathBuf,
}

impl InputDirectory {
    pub fn new(path: PathBuf) -> Self {
        InputDirectory { path }
    }
}

impl JsonSource for InputDirectory {
    fn get_entries(&self, sort: bool) -> Vec<(String, Value)> {
        read_entries(&self.path, sort).expect("Error reading entries from directory")
    }

    fn read_entry(&self) -> Result<(String, Value)> {
        let file = &self.path;
        log::info!("Reading file {}", &file.display());
        let object = read_object(file)?;
        let name = file.file_stem().unwrap().to_str().unwrap();
        Ok((name.to_string(), object))
    }
}

impl AsRef<PathBuf> for InputDirectory {
    fn as_ref(&self) -> &PathBuf {
        &self.path
    }
}

impl std::str::FromStr for InputDirectory {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Ok(InputDirectory::new(PathBuf::from(input)))
    }
}

pub fn read_entries(dir: &PathBuf, sort: bool) -> Result<Vec<(String, Value)>> {
    let mut entries: Vec<(String, Value)> = Vec::new();
    let dir_path = PathBuf::from(dir);
    let dir_entries = std::fs::read_dir(dir_path)?;
    for entry in dir_entries {
        if let Ok(entry) = entry {
            let file_path = entry.path();
            let object = read_object(&file_path)?;
            let name = file_path.file_stem().unwrap().to_str().unwrap();
            log::info!("Appending entry {} from {:?}", name, file_path);
            entries.push((name.to_string(), object));
        }
    }

    if sort {
        entries.sort_by(|a, b| a.0.cmp(&b.0));
    }

    Ok(entries)
}
