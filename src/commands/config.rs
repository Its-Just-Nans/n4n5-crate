//! Config subcommand
use std::process::Command;

use clap::{ArgAction, Subcommand};

use crate::{config::Config, errors::GeneralError};

/// Config subcommand
#[derive(Subcommand, Debug)]
pub enum ConfigSubcommand {
    /// Open config with default editor
    Open {
        /// Print the path
        #[arg(short = 'p', long = "path", action = ArgAction::SetTrue)]
        show_path_only: bool,
    },
}
impl ConfigSubcommand {
    /// invoke subcommand
    /// # Errors
    /// Error if error in subcommand
    pub fn invoke(self, config: &mut Config) -> Result<(), GeneralError> {
        match self {
            ConfigSubcommand::Open { show_path_only } => Config::open(config, show_path_only)?,
        };
        Ok(())
    }
}
impl Config {
    /// Open the config file with the default editor
    /// # Errors
    /// Return an error if the editor fails to open
    fn open(config: &mut Config, print_path: bool) -> Result<(), GeneralError> {
        let config_path = &config.config_path;
        if print_path {
            println!("{}", config_path.display());
            return Ok(());
        }
        println!("Opening config {}", config_path.display());
        Command::new("vi").arg(config_path).spawn()?.wait()?;
        Ok(())
    }
}
