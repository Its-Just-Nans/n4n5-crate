//! To see all subcommands, run:
//!
//! ```shell
//! n4n5 utils
//! ```
//!

use clap::{ArgMatches, Command as ClapCommand};
use serde::{Deserialize, Serialize};

use crate::{cli::CliCommand, config::Config, errors::GeneralError};

/// Movies configuration
#[derive(Deserialize, Serialize, Default)]
pub struct UtilsCliCommand {}

impl CliCommand for UtilsCliCommand {
    fn get_subcommand() -> ClapCommand {
        ClapCommand::new("utils")
            .about("utils")
            .subcommand(Self::list_crates_args())
            .arg_required_else_help(true)
    }
    fn invoke(config: &mut Config, args_matches: &ArgMatches) -> Result<(), GeneralError> {
        if let Some(matches) = args_matches.subcommand_matches("list_crates") {
            UtilsCliCommand::list_crates(config, matches)?;
        }
        Ok(())
    }
}
