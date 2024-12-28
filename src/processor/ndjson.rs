use super::json_field::JsonField;
use crate::{
    input::{InputDirectory, JsonReaderInput, JsonSource},
    output::Output,
};
use serde_json::Value;

pub struct NdjsonBundler {
    input: InputDirectory,
    output: Output,
}

impl NdjsonBundler {
    pub fn new(input: InputDirectory, output: Output) -> Self {
        Self { input, output }
    }

    /// Bundles JSON files from the specified directory into a single output.
    ///
    /// # Arguments
    ///
    /// * `dir` - A reference to a `PathBuf` representing the directory containing JSON files.
    /// * `output` - A reference to an `Output` where the bundled JSON will be written.

    pub fn bundle(&self, json_fields: Option<Vec<String>>) -> std::io::Result<()> {
        if let Output::Directory { .. } = self.output {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Cannot bundle to a directory",
            ));
        }
        self.read_entries_to_output(json_fields)
    }

    /// Reads all JSON files in the specified directory and appends their contents to the output.
    ///
    /// # Arguments
    ///
    /// * `dir` - A reference to a `PathBuf` representing the directory containing JSON files.
    /// * `output` - A reference to an `Output` where the JSON data will be appended.
    ///
    /// # Errors
    ///
    /// Returns an `std::io::Error` if any file cannot be processed or if reading fails.

    fn read_entries_to_output(&self, json_fields: Option<Vec<String>>) -> std::io::Result<()> {
        log::debug!("Escaping fields: {:?}", json_fields);
        self.input
            .get_entries(false)
            .drain(..)
            .map(|(_name, mut json)| {
                if let Some(ref json_fields) = json_fields {
                    json_fields.iter().for_each(|field| {
                        if let Some(value) = json.pointer_mut(&dots_to_slashes(field)) {
                            log::debug!("Escaping field {}", field);
                            *value = JsonField::from(value.clone()).escape();
                        }
                    });
                }
                self.output.append(json)
            })
            .collect()
    }
}

pub struct NdjsonUnbundler {
    input: JsonReaderInput,
    output: Output,
}

impl NdjsonUnbundler {
    pub fn new(input: JsonReaderInput, output: Output) -> Self {
        Self { input, output }
    }

    /// Unbundles NDJSON file and writes separate JSON files to the specified output.
    ///
    /// # Arguments
    ///
    /// * `input` - A refeence to an `Input` representing the source of NDJSON data.
    /// * `output` - A reference to an `Output` where the JSON files will be written.
    /// * `name` - An optional name for the JSON objects, used as a key to extract values.

    pub fn unbundle(
        &self,
        name: Option<Vec<String>>,
        type_field: Option<String>,
        unescape_fields: Option<Vec<String>>,
    ) -> std::io::Result<()> {
        let mut i: usize = 0;
        let name_list = match name {
            Some(list) => list
                .iter()
                .map(|name| dots_to_slashes(&name))
                .collect::<Vec<String>>(),
            None => vec![],
        };
        let type_field = type_field.map(|field| dots_to_slashes(&field));

        let name_entry = |i: usize, json: &Value| {
            let default_name = format!("object-{i:06}");

            let name = name_list
                .iter()
                .find_map(|name| json.pointer(name))
                .map_or(default_name, |value| {
                    value.as_str().unwrap_or_default().to_string()
                });

            match &type_field {
                Some(field) => json.pointer(&field).map_or(name.clone(), |value| {
                    format!("{name}.{}", value.as_str().unwrap_or_default().to_string())
                }),
                None => name,
            }
        };

        let mut buf = String::new();
        while let Ok(()) = self.input.read_line(&mut buf) {
            match serde_json::from_str::<Value>(&buf) {
                Ok(mut json) => {
                    if let Some(ref unescape_fields) = unescape_fields {
                        unescape_fields.iter().for_each(|field| {
                            if let Some(value) = json.pointer_mut(&dots_to_slashes(field)) {
                                log::debug!("Unescaping field {}", field);
                                *value = JsonField::from(value.clone()).unescape();
                            }
                        });
                    }
                    let entry = vec![(name_entry(i, &json), json)];
                    self.output.write_entries(entry)?
                }
                Err(e) if serde_json::Error::is_eof(&e) => break,
                Err(e) => log::error!("Failed to parse line {}: {}", i, e),
            }
            buf.clear();
            i += 1;
        }
        Ok(())
    }
}

fn dots_to_slashes(str: &str) -> String {
    "/".to_string() + &str.split('.').collect::<Vec<&str>>().join("/")
}
