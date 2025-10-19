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
}

impl UtilsSubCommand {
    /// invoke the subcommand
    /// # Errors
    /// Error if error in subcommand
    pub fn invoke(self, config: &mut Config) -> Result<(), GeneralError> {
        match self {
            UtilsSubCommand::ListCrates(subcommand) => {
                UtilsListCrates::list_crates(config, subcommand)
            }
        }
    }
}
