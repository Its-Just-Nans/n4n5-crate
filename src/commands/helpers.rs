//! To see all subcommands, run:
//!
//! ```shell
//! n4n5 helpers
//! ```
//!

use std::{
    fs::{create_dir_all, File},
    path::Path,
};

use clap::{ArgMatches, Command as ClapCommand};
use serde::{Deserialize, Serialize};

use clap_complete::{
    generate_to,
    shells::{Bash, Elvish, Fish, PowerShell, Zsh},
};
use clap_mangen::Man;

use crate::{
    cli::{get_cli_args, CliCommand},
    config::Config,
    errors::GeneralError,
};

/// Movies configuration
#[derive(Deserialize, Serialize, Default)]
pub struct HelpersCliCommand {}

impl CliCommand for HelpersCliCommand {
    fn get_subcommand() -> ClapCommand {
        ClapCommand::new("helpers")
            .about("helpers")
            .subcommand(ClapCommand::new("completions").about("generate completions"))
            .subcommand(ClapCommand::new("man").about("generate man"))
            .arg_required_else_help(true)
    }
    fn invoke(config: &mut Config, args_matches: &ArgMatches) -> Result<(), GeneralError> {
        if let Some(matches) = args_matches.subcommand_matches("completions") {
            HelpersCliCommand::gen_completions(config, matches)?;
        } else if let Some(matches) = args_matches.subcommand_matches("man") {
            HelpersCliCommand::gen_man(config, matches)?;
        }
        Ok(())
    }
}

impl HelpersCliCommand {
    /// Get the music file path
    /// # Errors
    /// Fails if the file cannot be found
    pub fn gen_completions(
        _config: &mut Config,
        _matches: &ArgMatches,
    ) -> Result<(), GeneralError> {
        let mut cmd = get_cli_args();
        let app_name = env!("CARGO_CRATE_NAME");
        let outdir = Path::new("completions");

        create_dir_all(outdir)?;

        generate_to(Bash, &mut cmd, app_name, outdir)?;
        generate_to(Zsh, &mut cmd, app_name, outdir)?;
        generate_to(Fish, &mut cmd, app_name, outdir)?;
        generate_to(PowerShell, &mut cmd, app_name, outdir)?;
        generate_to(Elvish, &mut cmd, app_name, outdir)?;

        Ok(())
    }

    /// generate man page
    /// # Errors
    /// Fails if error
    pub fn gen_man(_config: &mut Config, _matches: &ArgMatches) -> Result<(), GeneralError> {
        let cmd = get_cli_args();
        let app_name = env!("CARGO_CRATE_NAME");
        let filename = format!("{}.1", app_name);
        let mut file = File::create(filename)?;

        Man::new(cmd).render(&mut file)?;
        Ok(())
    }
}
