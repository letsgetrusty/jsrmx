use super::{AllOutputs, FileAndStdOut};
use serde::Serialize;
use serde_json::Value;
use std::io::stdout;

#[derive(Clone, Debug)]
pub struct StreamOutput {
    pretty: bool,
}

impl StreamOutput {
    pub fn new(pretty: bool) -> Self {
        Self { pretty }
    }
}

impl FileAndStdOut for StreamOutput {
    fn append<T: Serialize>(&self, content: T) -> std::io::Result<()> {
        match self.pretty {
            true => serde_json::to_writer_pretty(stdout(), &content)?,
            false => serde_json::to_writer(stdout(), &content)?,
        }
        println!();
        Ok(())
    }

    fn write<T: Serialize>(&self, content: T) -> std::io::Result<()> {
        match self.pretty {
            true => serde_json::to_writer_pretty(stdout(), &content)?,
            false => serde_json::to_writer(stdout(), &content)?,
        }
        Ok(())
    }
}

impl AllOutputs for StreamOutput {
    fn set_pretty(&mut self, pretty: bool) {
        self.pretty = pretty;
    }

    fn write_entries(&self, mut entries: Vec<(String, Value)>) -> std::io::Result<()> {
        for (key, value) in entries.drain(..) {
            let entry = serde_json::json!({key: value});
            match self.pretty {
                true => serde_json::to_writer_pretty(stdout(), &entry)?,
                false => serde_json::to_writer(stdout(), &entry)?,
            }
            println!();
        }
        Ok(())
    }
}
