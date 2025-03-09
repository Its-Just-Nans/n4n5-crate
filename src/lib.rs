//! # n4n5
//!
//! n4n5 is a CLI utility with subcommands
//!
//! To see all subcommands, run:
//! ```shell
//! n4n5
//! ```

#![deny(
    missing_docs,
    clippy::all,
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::cargo
)]
#![warn(clippy::multiple_crate_versions)]

pub(crate) mod cli;
pub(crate) mod commands;
pub mod config;
pub(crate) mod errors;
pub(crate) mod macros;
pub(crate) use macros::config_path;
pub(crate) use macros::config_sub_path;

pub use cli::cli_main;
