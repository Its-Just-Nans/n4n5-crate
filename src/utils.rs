//! Utils functions

use std::path::Path;

use serde::Serialize;
use std::fs::write;

/// Write date to a file, with pretty json
/// # Errors
/// Fails if serilize fails or wirte fail
pub(crate) fn serde_pretty_print<T>(
    data: T,
    path_file: &Path,
    print_json: bool,
) -> Result<(), std::io::Error>
where
    T: Serialize,
{
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut buf = Vec::new();
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    data.serialize(&mut ser)?;
    if print_json {
        println!("{}", String::from_utf8_lossy(&buf));
    }
    write(path_file, buf)?;
    Ok(())
}
