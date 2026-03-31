//! JSON output for debugging and inspection

use std::io::Write;
use std::path::Path;
use crate::error::Result;
use crate::packets::dp_container::DpContainer;

/// Write a DpContainer to JSON file
pub fn write_dp_container_json<P: AsRef<Path>>(
    container: &DpContainer,
    path: P,
    pretty: bool,
) -> Result<()> {
    let json = if pretty {
        serde_json::to_string_pretty(container)?
    } else {
        serde_json::to_string(container)?
    };

    let mut file = std::fs::File::create(path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

/// Convert a DpContainer to JSON string
pub fn dp_container_to_json(container: &DpContainer, pretty: bool) -> Result<String> {
    let json = if pretty {
        serde_json::to_string_pretty(container)?
    } else {
        serde_json::to_string(container)?
    };
    Ok(json)
}
