use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::{fs, io};

/// Reads a JSON object from the specified file path.
///
/// # Arguments
///
/// * `input` - A reference to a `PathBuf` that represents the path to the JSON file.
///
/// # Returns
///
/// Returns an `io::Result<Value>` which contains the parsed JSON object on success,
/// or an error if the file cannot be read or the content is not valid JSON.
///
/// # Example
///
/// ```
/// let path = PathBuf::from("data.json");
/// match read_object(&path) {
///     Ok(json) => println!("Successfully read JSON: {:?}", json),
///     Err(e) => eprintln!("Error reading JSON: {}", e),
/// }
/// ```

pub fn read_object(input: &PathBuf) -> io::Result<Value> {
    let file = File::open(input)?;
    let reader = BufReader::new(file);
    let json_value = serde_json::from_reader(reader)?;
    Ok(json_value)
}

/// Reads a JSON object from the specified file path and returns it as a HashMap. Usefull for
/// non-deterministicly iterating over large objects in parallel.
///
/// # Arguments
///
/// * `input` - A reference to a `PathBuf` that represents the path to the JSON file.
///
/// # Returns
///
/// Returns an `io::Result<HashMap<String, Value>>` which contains the parsed JSON object as a HashMap
/// on success, or an error if the file cannot be read or the content is not valid JSON.
///
/// # Example
///
/// ```
/// let path = PathBuf::from("data.json");
/// match read_hashmap(&path) {
///     Ok(hashmap) => println!("Successfully read hashmap: {:?}", hashmap),
///     Err(e) => eprintln!("Error reading entries: {}", e),
/// }
/// ```

pub fn read_hashmap(input: &PathBuf) -> io::Result<HashMap<String, Value>> {
    let file = File::open(input)?;
    let reader = BufReader::new(file);
    let hashmap: HashMap<String, Value> = serde_json::from_reader(reader)?;
    Ok(hashmap)
}

/// Reads JSON objects from all files in the specified directory and returns them as a vector of tuples.
///
/// # Arguments
///
/// * `input` - A reference to an `Input` enum that specifies the directory to read from.
/// * `_extension` - An optional string slice that can be used to filter files by extension (currently unused).
/// * `sort` - A boolean indicating whether to sort the returned entries by their names.
///
/// # Returns
///
/// Returns an `io::Result<Vec<(String, Value)>>` which contains a vector of tuples, where each tuple
/// consists of the filename (without extension) and the corresponding parsed JSON object. If the
/// directory cannot be read or any file cannot be parsed as JSON, it returns an error.
///
/// # Example
///
/// ```
/// let input = Input::Directory("path/to/directory".into());
/// match read_entries_from_directory(&input, None, true) {
///     Ok(entries) => println!("Successfully read entries: {:?}", entries),
///     Err(e) => eprintln!("Error reading entries: {}", e),
/// }
/// ```

pub fn read_entries_from_directory(dir: &PathBuf, sort: bool) -> io::Result<Vec<(String, Value)>> {
    let mut entries: Vec<(String, Value)> = Vec::new();
    let dir_path = PathBuf::from(dir);
    let dir_entries = fs::read_dir(dir_path)?;
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
