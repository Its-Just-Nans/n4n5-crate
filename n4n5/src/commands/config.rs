//! config command

use clap::{ArgAction, Subcommand};
use std::process::Command;

use crate::config::Config;
use crate::errors::GeneralError;

/// Config subcommand
#[derive(Subcommand, Debug, Clone)]
pub enum ConfigSubcommand {
    /// Open config with default editor
    Open {
        /// Print the path
        #[arg(short = 'p', long = "path", action = ArgAction::SetTrue)]
        show_path_only: bool,
    },
}

impl ConfigSubcommand {
    /// Invoke subcommands
    /// # Errors
    /// Fails if subcommand fails
    pub(crate) fn invoke(&self, config: &mut Config) -> Result<(), GeneralError> {
        match self {
            ConfigSubcommand::Open { show_path_only } => {
                ConfigSubcommand::open(config, *show_path_only)
            }
        }
    }

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
        let editor = std::env::var("EDITOR").unwrap_or("vi".to_string());
        Command::new(editor).arg(config_path).spawn()?.wait()?;
        Ok(())
    }
}
