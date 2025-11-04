//! Utils functions

use std::{fmt::Write, path::Path};

use serde::Serialize;
use std::fs::write;

/// Write date to a file, with pretty json
/// # Errors
/// Fails if serialize fails or write fails
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

/// Format a table to markdown
/// # Errors
/// Fails if fmt error
pub fn table_to_markdown_table<I>(table: I) -> Result<String, std::fmt::Error>
where
    I: Iterator<Item = Vec<String>> + Clone,
{
    let mut buf = String::new();
    let max_sizes = table.clone().fold([0usize; 3], |mut acc, row| {
        for (i, cell) in row.iter().enumerate() {
            acc[i] = acc[i].max(cell.len());
        }
        acc
    });

    for (i, row) in table.enumerate() {
        let line = row
            .iter()
            .zip(max_sizes)
            .map(|(s, width)| format!("{:width$}", s, width = width))
            .collect::<Vec<_>>()
            .join(" | ");
        writeln!(&mut buf, "| {} |", line)?;

        // separator after header
        if i == 0 {
            let sep = max_sizes
                .iter()
                .map(|&w| "-".repeat(w))
                .collect::<Vec<_>>()
                .join(" | ");
            writeln!(&mut buf, "| {} |", sep)?;
        }
    }
    Ok(buf)
}
