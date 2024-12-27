use super::json_field::JsonField;
use crate::{
    input::{InputDirectory, JsonReaderInput},
    output::Output,
};
use serde_json::Value;
use std::path::PathBuf;

/// Bundles JSON files from the specified directory into a single output.
///
/// # Arguments
///
/// * `dir` - A reference to a `PathBuf` representing the directory containing JSON files.
/// * `output` - A reference to an `Output` where the bundled JSON will be written.

pub fn bundle(
    input: &InputDirectory,
    output: &Output,
    escape_fields: Vec<String>,
) -> std::io::Result<()> {
    if let Output::Directory { .. } = output {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Cannot bundle to a directory",
        ));
    }
    read_directory_to_output(input.as_ref(), output, escape_fields)
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
    input: &JsonReaderInput,
    output: &Output,
    name: Option<Vec<String>>,
    type_field: Option<String>,
    unescape_fields: Vec<String>,
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
    while let Ok(_) = input.read_line(&mut buf) {
        match serde_json::from_str::<Value>(&buf) {
            Ok(mut json) => {
                unescape_fields.iter().for_each(|field| {
                    if let Some(value) = json.pointer_mut(&dots_to_slashes(field)) {
                        log::debug!("Unescaping field {}", field);
                        *value = JsonField::from(value.clone()).unescape();
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
