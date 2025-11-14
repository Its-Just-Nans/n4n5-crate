//! To see all subcommands, run:
//!
//! ```shell
//! n4n5 helpers
//! ```
//!

use clap::{CommandFactory, Subcommand};
use clap_complete::{
    generate_to,
    shells::{Bash, Elvish, Fish, PowerShell, Zsh},
};
use clap_mangen::generate_to as man_generate_to;
use home::home_dir;
use std::fs::create_dir_all;

use crate::{cli::CliArgs, config::Config, errors::GeneralError};

/// Movies configuration
#[derive(Subcommand, Debug)]
pub enum HelpersSubcommand {
    /// generate completions
    Completions,
    /// generate man
    Man,
}

impl HelpersSubcommand {
    /// invoke subcommand
    /// # Errors
    /// Error if error in subcommand
    pub fn invoke(self, config: &mut Config) -> Result<(), GeneralError> {
        match self {
            HelpersSubcommand::Completions => Self::gen_completions(config),
            HelpersSubcommand::Man => Self::gen_man(config),
        }
    }
    /// Get the music file path
    /// # Errors
    /// Fails if the file cannot be found
    pub fn gen_completions(_config: &mut Config) -> Result<(), GeneralError> {
        let mut cmd = CliArgs::command();
        let app_name = env!("CARGO_CRATE_NAME");
        let outdir = home_dir().ok_or(GeneralError::new("Cannot get home dir"))?;
        let outdir = outdir.join(".config").join(".n4n5").join("completions");

        create_dir_all(&outdir)?;
        generate_to(Bash, &mut cmd, app_name, &outdir)?;
        generate_to(Zsh, &mut cmd, app_name, &outdir)?;
        generate_to(Fish, &mut cmd, app_name, &outdir)?;
        generate_to(PowerShell, &mut cmd, app_name, &outdir)?;
        generate_to(Elvish, &mut cmd, app_name, &outdir)?;

        Ok(())
    }

    /// generate man page
    /// # Errors
    /// Fails if error
    pub fn gen_man(_config: &mut Config) -> Result<(), GeneralError> {
        let cmd = CliArgs::command();
        let outdir = home_dir().ok_or(GeneralError::new("Cannot get home dir"))?;
        let outdir = outdir.join(".config").join(".n4n5").join("man");
        create_dir_all(&outdir)?;

        man_generate_to(cmd, outdir)?;
        Ok(())
    }
}
