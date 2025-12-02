//! Utils functions

use serde::Serialize;
use std::{
    fs::write,
    path::{Path, PathBuf},
};

use crate::errors::GeneralError;

/// Write date to a file, with pretty json
/// # Errors
/// Fails if serialize fails or write fails
pub fn pretty_print<T>(data: T, path_file: &Path) -> Result<(), std::io::Error>
where
    T: Serialize,
{
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut buf = Vec::new();
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    data.serialize(&mut ser)?;
    if path_file == "-" {
        println!("{}", String::from_utf8_lossy(&buf));
    } else {
        write(path_file, buf)?;
    }
    Ok(())
}

/// Format a table to markdown
/// # Errors
/// Fails if fmt error
pub fn table_to_markdown_table<I>(table: I, columns: usize) -> Result<String, std::fmt::Error>
where
    I: Iterator<Item = Vec<String>> + Clone,
{
    use core::fmt::Write;
    let mut buf = String::new();
    let max_sizes = table.clone().fold(vec![0; columns], |mut acc, row| {
        for (i, cell) in row.iter().enumerate() {
            acc[i] = acc[i].max(cell.len());
        }
        acc
    });

    for (i, row) in table.enumerate() {
        let line = row
            .iter()
            .zip(&max_sizes)
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

/// Get input from the user with a prompt
/// # Errors
/// Returns a GeneralError if the input fails
pub fn get_input(text: &str) -> Result<String, GeneralError> {
    println!("{text}");
    input()
}

/// Get input from the user
/// # Errors
/// Returns a GeneralError if the input fails
pub fn input() -> Result<String, GeneralError> {
    use std::io::{Write, stdin, stdout};
    let mut s = String::new();
    let _ = stdout().flush();
    stdin()
        .read_line(&mut s)
        .map_err(|e| GeneralError::new(format!("Failed to read line from stdin:{}", e)))?;
    if let Some('\n') = s.chars().next_back() {
        s.pop();
    }
    if let Some('\r') = s.chars().next_back() {
        s.pop();
    }
    Ok(s)
}

/// Get a yes input from the user
/// # Errors
/// Returns a GeneralError if the input fails
pub fn input_yes<S: AsRef<str>>(prompt: S) -> Result<bool, GeneralError> {
    use std::io::Write;
    print!("{} (y/n):", prompt.as_ref());
    std::io::stdout().flush()?;
    let s = input()?;
    Ok(matches!(s.to_lowercase().as_str(), "y" | "yes"))
}

/// Get a no input from the user
/// # Errors
/// Returns a GeneralError if the input fails
pub fn input_no<S: AsRef<str>>(prompt: S) -> Result<bool, GeneralError> {
    let input_y = input_yes(prompt)?;
    Ok(!input_y)
}

/// Get a valid path from the user
/// # Errors
/// Returns a GeneralError if the path does not exist
pub fn input_path() -> Result<(PathBuf, String), GeneralError> {
    let mut s = input()?;
    let mut path = PathBuf::from(&s);
    loop {
        if s == "\\" {
            return Err(GeneralError::new("no path"));
        }
        if path.exists() {
            break;
        }
        println!("Path does not exist. Please enter a valid path:");
        s = input()?;
        path = PathBuf::from(&s);
    }
    let path_to_string = path.to_string_lossy();
    Ok((path.clone(), path_to_string.to_string()))
}
