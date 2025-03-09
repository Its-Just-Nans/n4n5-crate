//! Error handling for the application

use std::{
    num::{ParseFloatError, ParseIntError},
    string::FromUtf8Error,
};

/// General error type for the application
pub struct GeneralError {
    /// Error message
    message: String,
}

impl GeneralError {
    /// Create a new GeneralError instance
    pub fn new(message: String) -> GeneralError {
        GeneralError { message }
    }
}

impl std::fmt::Display for GeneralError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "GeneralError: {}", self.message)
    }
}

impl From<std::io::Error> for GeneralError {
    fn from(error: std::io::Error) -> Self {
        GeneralError::new(format!("IO Error: {}", error))
    }
}

// serde_json::Error is a type alias for serde_json::error::Error
impl From<serde_json::Error> for GeneralError {
    fn from(error: serde_json::Error) -> Self {
        GeneralError::new(format!("JSON Error: {}", error))
    }
}

impl From<ParseIntError> for GeneralError {
    fn from(value: ParseIntError) -> Self {
        GeneralError::new(format!("ParseIntError: {}", value))
    }
}

impl From<ParseFloatError> for GeneralError {
    fn from(value: ParseFloatError) -> Self {
        GeneralError::new(format!("ParseIntError: {}", value))
    }
}

impl From<FromUtf8Error> for GeneralError {
    fn from(value: FromUtf8Error) -> Self {
        GeneralError::new(format!("FromUtf8Error: {}", value))
    }
}

impl From<toml::ser::Error> for GeneralError {
    fn from(value: toml::ser::Error) -> Self {
        GeneralError::new(format!("Toml de Error: {}", value))
    }
}
