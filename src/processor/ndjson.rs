use super::json_field::JsonField;
use crate::{input::Input, output::Output};
use serde_json::Value;
use std::path::PathBuf;

/// Bundles JSON files from the specified directory into a single output.
///
/// # Arguments
///
/// * `dir` - A reference to a `PathBuf` representing the directory containing JSON files.
/// * `output` - A reference to an `Output` where the bundled JSON will be written.

pub fn bundle(input: &Input, output: &Output, escape_fields: Vec<String>) -> std::io::Result<()> {
    if let Output::Directory { .. } = output {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Cannot bundle to a directory",
        ));
    }
    match input {
        Input::Directory(dir) => read_directory_to_output(dir, output, escape_fields),
        Input::File { .. } => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Cannot bundle from a single file, multiple objects in a file is invalid JSON!",
            ))
        }
        Input::Stdin { .. } => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Why bundle from stdin? Just redirect output to a file!",
            ))
        }
    }
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

fn read_directory_to_output(
    dir: &PathBuf,
    output: &Output,
    json_fields: Vec<String>,
) -> std::io::Result<()> {
    let files = std::fs::read_dir(dir)?;
    log::debug!("Escaping fields: {:?}", json_fields);
    for file in files {
        let file = file?.path();
        log::info!("Reading file {}", &file.display());
        if file.extension().and_then(|s| s.to_str()) == Some("json") {
            let content = std::fs::read_to_string(&file)?;
            let mut json: Value = serde_json::from_str(&content)?;
            json_fields
                .iter()
                .for_each(|field| match json.pointer_mut(&dots_to_slashes(field)) {
                    Some(value) => {
                        log::debug!("Escaping field {}", field);
                        *value = JsonField::from(value.clone()).escape();
                    }
                    None => (),
                });
            output.append(json)?
        }
    }
    Ok(())
}

/// Unbundles NDJSON file and writes separate JSON files to the specified output.
///
/// # Arguments
///
/// * `input` - A refeence to an `Input` representing the source of NDJSON data.
/// * `output` - A reference to an `Output` where the JSON files will be written.
/// * `name` - An optional name for the JSON objects, used as a key to extract values.

pub fn unbundle(
    input: &Input,
    output: &Output,
    name: Option<&str>,
    unescape_fields: Vec<String>,
) -> std::io::Result<()> {
    let mut i: usize = 0;

    let name_entry = |i: usize, json: &Value| {
        let default_name = format!("object-{i:06}");
        match name {
            Some(name) => json
                .pointer(&dots_to_slashes(&name))
                .and_then(|value| value.as_str())
                .unwrap_or(&default_name)
                .to_string(),
            None => default_name,
        }
    };

    let mut buf = String::new();
    while let Ok(_) = input.read_line(&mut buf) {
        match serde_json::from_str::<Value>(&buf) {
            Ok(mut json) => {
                unescape_fields.iter().for_each(|field| {
                    match json.pointer_mut(&dots_to_slashes(field)) {
                        Some(value) => {
                            log::debug!("Unescaping field {}", field);
                            *value = JsonField::from(value.clone()).unescape();
                        }
                        None => (),
                    }
                });
                let entry = vec![(name_entry(i, &json), json)];
                output.write_entries(entry)?
            }
            Err(e) if serde_json::Error::is_eof(&e) => break,
            Err(e) => log::error!("Failed to parse line {}: {}", i, e),
        }
        buf.clear();
        i += 1;
    }
    Ok(())
}

fn dots_to_slashes(str: &str) -> String {
    "/".to_string() + &str.split('.').collect::<Vec<&str>>().join("/")
}
