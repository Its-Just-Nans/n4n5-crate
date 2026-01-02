//! Error handling for the application

use std::{
    num::{ParseFloatError, ParseIntError},
    string::FromUtf8Error,
};

/// General error type for the application
#[derive(Debug)]
pub struct GeneralError {
    /// Error message
    message: String,
    /// Error source
    from: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl GeneralError {
    /// Create a new [`GeneralError`] instance
    pub fn new<S: AsRef<str>>(msg: S) -> Self {
        let message = msg.as_ref().to_string();
        Self {
            message,
            from: None,
        }
    }

    /// Create a new [`GeneralError`] instance with a source
    pub fn new_with_source<S: Into<String>, B: std::error::Error + Send + Sync + 'static>(
        message: S,
        from: B,
    ) -> Self {
        Self {
            message: message.into(),
            from: Some(Box::new(from)),
        }
    }
}

impl std::error::Error for GeneralError {}

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
    fn from(value: std::io::Error) -> Self {
        Self::new_with_source(value.to_string(), value)
    }
}

impl From<&str> for GeneralError {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for GeneralError {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

// serde_json::Error is a type alias for serde_json::error::Error
impl From<serde_json::Error> for GeneralError {
    fn from(value: serde_json::Error) -> Self {
        Self::new_with_source(value.to_string(), value)
    }
}

impl From<ParseIntError> for GeneralError {
    fn from(value: ParseIntError) -> Self {
        Self::new(format!("ParseIntError: {value}"))
    }
}

impl From<ParseFloatError> for GeneralError {
    fn from(value: ParseFloatError) -> Self {
        Self::new(format!("ParseFloatError: {value}"))
    }
}

impl From<FromUtf8Error> for GeneralError {
    fn from(value: FromUtf8Error) -> Self {
        Self::new_with_source(value.to_string(), value)
    }
}

impl From<toml::ser::Error> for GeneralError {
    fn from(value: toml::ser::Error) -> Self {
        Self::new_with_source(value.to_string(), value)
    }
}

impl From<reqwest::Error> for GeneralError {
    fn from(value: reqwest::Error) -> Self {
        Self::new_with_source(value.to_string(), value)
    }
}

impl From<std::fmt::Error> for GeneralError {
    fn from(value: std::fmt::Error) -> Self {
        Self::new_with_source(value.to_string(), value)
    }
}

impl From<toml::de::Error> for GeneralError {
    fn from(value: toml::de::Error) -> Self {
        Self::new_with_source(value.to_string(), value)
    }
}

impl From<clap::error::Error> for GeneralError {
    fn from(value: clap::error::Error) -> Self {
        Self::new_with_source(value.to_string(), value)
    }
}

impl<S, B> From<(S, B)> for GeneralError
where
    S: Into<String>,
    B: std::error::Error + Send + Sync + 'static,
{
    fn from(value: (S, B)) -> Self {
        // value.0 is the string, value.1 is the error
        Self::new_with_source(value.0.into(), value.1)
    }
}
