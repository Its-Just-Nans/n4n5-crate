//! n4n5-share lib
//! Quick, easy and dirty file transfer

#![warn(clippy::all, rust_2018_idioms)]
#![deny(
    missing_docs,
    clippy::all,
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::cargo,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::pedantic
)]
#![allow(clippy::multiple_crate_versions)]

pub(crate) mod share;

pub use share::cli_main;
