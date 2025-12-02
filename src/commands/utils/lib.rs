//! To see all subcommands, run:
//!
//! ```shell
//! n4n5 utils
//! ```
//!
use clap::Subcommand;

use crate::{commands::utils::list_crates::UtilsListCrates, config::Config, errors::GeneralError};

/// Movies configuration
#[derive(Subcommand, Debug)]
pub enum UtilsSubCommand {
    /// list_crates subcommand
    #[command(name = "list_crates")]
    ListCrates(UtilsListCrates),

    /// Launch pngtools cli
    #[cfg(feature = "pngtools")]
    #[command(name = "pngtools")]
    PngTools,
}

impl UtilsSubCommand {
    /// invoke the subcommand
    /// # Errors
    /// Error if error in subcommand
    pub fn invoke(self, config: &mut Config) -> Result<(), GeneralError> {
        match self {
            UtilsSubCommand::ListCrates(subcommand) => subcommand.list_crates(config),
            #[cfg(feature = "pngtools")]
            UtilsSubCommand::PngTools => self.pngtools(),
        }
    }

    /// Run pngtools cli
    /// # Errors
    /// Fails if pngtools fails
    #[cfg(feature = "pngtools")]
    pub fn pngtools(&self) -> Result<(), GeneralError> {
        use std::sync::Arc;
        pngtools::run_cli().map_err(|e| {
            GeneralError::new_with_source(
                format!("Error with pngtools: {e}"),
                Box::new(Arc::new(e)),
            )
        })?;
        Ok(())
    }
}
