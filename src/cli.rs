//! The CLI module

use clap::{ArgAction, CommandFactory, Parser, Subcommand};
use clap_complete::{
    generate_to,
    shells::{Bash, Elvish, Fish, PowerShell, Zsh},
};
use clap_mangen::generate_to as man_generate_to;
use home::home_dir;
use std::{fs::create_dir_all, path::PathBuf, process::Command};

use crate::{
    commands::{
        gh::lib::GhSubCommand, movies::MoviesSubCommand, shortcuts::ShortcutsSubcommand,
        sync::SyncSubcommand, utils::lib::UtilsSubCommand,
    },
    config::Config,
    errors::GeneralError,
};

/// Example CLI using clap derive and subcommands
#[derive(Parser, Debug)]
#[command(name = "n4n5")]
#[command(about = "n4n5 CLI", long_about = None)]
pub struct CliArgs {
    /// Sets a custom config file
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,

    /// whether to use input for configuration
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub use_input: bool,

    /// Subcommands
    #[command(subcommand)]
    pub command: Commands,
}

/// Main commands enum
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// utils subcommand
    Utils {
        /// list of subcommands
        #[command(subcommand)]
        subcommand: UtilsSubCommand,
    },

    /// config subcommand
    Config {
        /// list of subcommands
        #[command(subcommand)]
        subcommand: ConfigSubcommand,
    },

    /// gh subcommand
    Gh {
        /// list of subcommands
        #[command(subcommand)]
        subcommand: GhSubCommand,
    },

    /// movies subcommand
    Movies {
        /// list of subcommands
        #[command(subcommand)]
        subcommand: MoviesSubCommand,
    },

    /// sync subcommand
    Sync {
        /// list of subcommands
        #[command(subcommand)]
        subcommand: SyncSubcommand,
    },

    /// generate completions
    Completions,

    /// Shortcuts subcommand
    #[command(alias = "s")]
    Shortcuts {
        /// list of subcommands
        #[command(subcommand)]
        subcommand: ShortcutsSubcommand,
    },

    /// generate man
    Man,
}

impl Commands {
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

    /// Invoke subcommands
    /// # Errors
    /// Fails if subcommand fails
    fn invoke(self, config: &mut Config) -> Result<(), GeneralError> {
        match self {
            Commands::Utils { subcommand } => subcommand.invoke(config),
            Commands::Config { subcommand } => subcommand.invoke(config),
            Commands::Gh { subcommand } => subcommand.invoke(config),
            Commands::Movies { subcommand } => subcommand.invoke(config),
            Commands::Sync { subcommand } => subcommand.invoke(config),
            Commands::Completions => Commands::gen_completions(config),
            Commands::Man => Commands::gen_man(config),
            Commands::Shortcuts { subcommand } => subcommand.run(),
        }
    }
}

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
    fn invoke(&self, config: &mut Config) -> Result<(), GeneralError> {
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
        Command::new("vi").arg(config_path).spawn()?.wait()?;
        Ok(())
    }
}

/// The CLI main function
/// Handle all arguments and invoke the correct command
/// # Errors
/// Returns a [`GeneralError`] if the command fails
pub fn cli_main() -> Result<(), GeneralError> {
    let cli_args = CliArgs::parse();
    let CliArgs {
        command,
        use_input,
        debug,
        config,
    } = cli_args;
    let mut config = Config::try_new(config, debug, use_input)?;
    command.invoke(&mut config)
}
