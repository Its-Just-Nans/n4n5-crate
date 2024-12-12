//! # n4n5

#![deny(
    missing_docs,
    clippy::all,
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::cargo
)]
#![warn(clippy::multiple_crate_versions)]

/// main CLI functions
pub(crate) mod cli;

/// Commands in cli
pub(crate) mod commands;

/// Configuration
pub(crate) mod config;

pub use cli::cli_main;
