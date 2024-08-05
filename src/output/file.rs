use serde_json::Value;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;

/// Writes a JSON object to the specified file path.
///
/// # Arguments
///
/// * `filename` - A reference to a `Path` that represents the path where the JSON object will be written.
/// * `object` - The `Value` representing the JSON object to be written.
/// * `pretty` - A boolean indicating whether to write the JSON in a pretty (human-readable) format or not.
///
/// # Returns
///
/// Returns an `io::Result<()>` which indicates success or failure of the write operation.
///
/// # Example
///
/// ```
/// let path = Path::new("output.json");
/// let json_object = json!({"key": "value"});
/// if let Err(e) = write_object(&path, json_object, true) {
///     eprintln!("Error writing JSON: {}", e);
/// }
/// ```

pub fn write_object(filename: &Path, object: Value, pretty: bool) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).create(true).open(filename)?;
    let body = if pretty {
        serde_json::to_string_pretty(&object)?
    } else {
        serde_json::to_string(&object)?
    };
    Ok(file.write_all(body.as_bytes())?)
}
