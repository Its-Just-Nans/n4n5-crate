//! Error handling for the application

use std::{
    num::{ParseFloatError, ParseIntError},
    string::FromUtf8Error,
};

/// General error type for the application
pub struct GeneralError {
    /// Error message
    message: String,
    /// Error source
    from: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl GeneralError {
    /// Create a new GeneralError instance
    pub fn new(message: String) -> GeneralError {
        GeneralError {
            message,
            from: None,
        }
    }

    /// Create a new GeneralError instance with a source
    pub fn new_with_source(
        message: String,
        from: Box<dyn std::error::Error + Send + Sync>,
    ) -> GeneralError {
        GeneralError {
            message,
            from: Some(from),
        }
    }
}

impl std::fmt::Display for GeneralError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(from) = &self.from {
            write!(f, "GeneralError: {} from {}", self.message, from)
        } else {
            write!(f, "GeneralError: {}", self.message)
        }
    }
}

impl From<std::io::Error> for GeneralError {
    fn from(error: std::io::Error) -> Self {
        GeneralError::new_with_source("IO Error".to_string(), error.into())
    }
}

impl From<String> for GeneralError {
    fn from(error: String) -> Self {
        GeneralError::new_with_source("Error:".to_string(), error.into())
    }
}

// serde_json::Error is a type alias for serde_json::error::Error
impl From<serde_json::Error> for GeneralError {
    fn from(error: serde_json::Error) -> Self {
        GeneralError::new_with_source("Serde JSON Error".to_string(), error.into())
    }
}

impl From<ParseIntError> for GeneralError {
    fn from(value: ParseIntError) -> Self {
        GeneralError::new(format!("ParseIntError: {value}"))
    }
}

impl From<ParseFloatError> for GeneralError {
    fn from(value: ParseFloatError) -> Self {
        GeneralError::new(format!("ParseFloatError: {value}"))
    }
}

impl From<FromUtf8Error> for GeneralError {
    fn from(value: FromUtf8Error) -> Self {
        GeneralError::new_with_source("FromUtf8Error".to_string(), value.into())
    }
}

impl From<toml::ser::Error> for GeneralError {
    fn from(value: toml::ser::Error) -> Self {
        GeneralError::new_with_source("Toml Ser Error".to_string(), value.into())
    }
}

impl From<reqwest::Error> for GeneralError {
    fn from(value: reqwest::Error) -> Self {
        GeneralError::new_with_source("Toml Ser Error".to_string(), value.into())
    }
}
