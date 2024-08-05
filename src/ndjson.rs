use crate::{Input, Output};
use serde_json::Value;
use std::io;
use std::path::PathBuf;

/// Bundles JSON files from the specified directory into a single output.
///
/// # Arguments
///
/// * `dir` - A reference to a `PathBuf` representing the directory containing JSON files.
/// * `output` - A reference to an `Output` where the bundled JSON will be written.

pub fn bundle(input: &Input, output: &Output) -> io::Result<()> {
    if let Output::Directory { .. } = output {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Cannot bundle to a directory",
        ));
    }
    match input {
        Input::Directory(dir) => read_directory_to_output(dir, output),
        Input::File { .. } => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Cannot bundle from a single file, multiple objects in a file is invalid JSON!",
            ))
        }
        Input::Stdin { .. } => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
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

fn read_directory_to_output(dir: &PathBuf, output: &Output) -> std::io::Result<()> {
    let files = std::fs::read_dir(dir)?;
    for file in files {
        let file = file?.path();
        log::info!("Reading file {}", &file.display());
        if file.extension().and_then(|s| s.to_str()) == Some("json") {
            let content = std::fs::read_to_string(&file)?;
            let json: Value = serde_json::from_str(&content)?;
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

pub fn unbundle(input: &Input, output: &Output, name: Option<&str>) -> io::Result<()> {
    let (pretty, path): (bool, Option<&PathBuf>) = match output {
        Output::Stdout { pretty } => (*pretty, None),
        Output::Directory { path, pretty } => (*pretty, Some(path)),
        Output::File { path, .. } => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Cannot unbundle to a file: {path:?}"),
            ))
        }
    };

    match path {
        Some(path) => {
            log::info!("Writing JSON objects to {path:?}");
            if !path.exists() {
                std::fs::create_dir_all(path)?;
            }
        }
        None => log::info!("Writing JSON objects to stdout"),
    };

    let mut i: usize = 0;
    let dots_to_slashes =
        |str: &str| "/".to_string() + &str.split('.').collect::<Vec<&str>>().join("/");

    let write_json = |i: usize, json: Value| {
        let default_name = format!("object-{i:06}");
        let new_name = match name {
            Some(name) => json
                .pointer(&dots_to_slashes(&name))
                .and_then(|value| value.as_str())
                .unwrap_or(&default_name),
            None => &default_name,
        };

        match path {
            Some(path) => {
                let filename = path.join(format!("{new_name}.json"));
                let _ = output.write(&filename, json);
                log::info!("Wrote object to {}", filename.display());
            }
            None if pretty => println!("{:#}", json),
            None => println!("{json}"),
        };
    };

    let mut buf = String::new();
    while let Ok(_) = input.read_line(&mut buf) {
        match serde_json::from_str::<Value>(&buf) {
            Ok(json) => write_json(i, json),
            Err(e) if serde_json::Error::is_eof(&e) => break,
            Err(e) => log::error!("Failed to parse line {}: {}", i, e),
        }
        buf.clear();
        i += 1;
    }
    Ok(())
}
