use super::{JsonAppender, JsonWriter};
use serde::Serialize;
use serde_json::Value;
use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
};

#[derive(Clone, Debug)]
pub struct FileOutput {
    pretty: bool,
    writer: Arc<Mutex<BufWriter<File>>>,
    pub path: PathBuf,
}

impl JsonAppender for FileOutput {
    fn append<T: Serialize>(&self, content: T) -> std::io::Result<()> {
        let mut guard = self.writer.lock().expect("Failed to get writer lock");
        match self.pretty {
            true => serde_json::to_writer_pretty(&mut *guard, &content)?,
            false => serde_json::to_writer(&mut *guard, &content)?,
        }
        writeln!(&mut *guard)?;
        Ok(())
    }
}

impl JsonWriter for FileOutput {
    fn set_pretty(&mut self, pretty: bool) {
        self.pretty = pretty;
    }

    fn write_entries(&self, entries: Vec<(String, Value)>) -> std::io::Result<()> {
        let mut guard = self.writer.lock().expect("Failed to get writer lock");
        for (key, value) in entries {
            let entry = serde_json::json!({key: value});
            match self.pretty {
                true => serde_json::to_writer_pretty(&mut *guard, &entry)?,
                false => serde_json::to_writer(&mut *guard, &entry)?,
            }
            writeln!(&mut *guard)?;
        }
        Ok(())
    }
}

impl FileOutput {
    pub fn new(path: PathBuf, pretty: bool) -> Self {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .expect("Failed to open file");
        let writer = Arc::new(Mutex::new(BufWriter::new(file)));
        Self {
            pretty,
            writer,
            path,
        }
    }
}
