//! The CLI module

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{
    generate_to,
    shells::{Bash, Elvish, Fish, PowerShell, Zsh},
};
use clap_mangen::generate_to as man_generate_to;
use home::home_dir;
use std::{fs::create_dir_all, path::PathBuf};

use crate::{
    commands::{
        config::ConfigSubcommand, gh::lib::GhSubCommand, movies::MoviesSubCommand,
        music::MusicSubcommand, sync::SyncSubcommand, utils::lib::UtilsSubCommand,
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
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,

    /// Subcommands
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// utils subcommand
    Utils {
        /// list of subcommands
        #[command(subcommand)]
        subcommand: UtilsSubCommand,
    },

    /// music subcommand
    Music {
        /// list of subcommands
        #[command(subcommand)]
        subcommand: MusicSubcommand,
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
}

/// The CLI main function
/// Handle all arguments and invoke the correct command
/// # Errors
/// Returns a GeneralError if the command fails
pub fn cli_main() -> Result<(), GeneralError> {
    let cli = CliArgs::parse();
    let mut config = match cli.config {
        Some(config_path) => Config::try_new_from_path(config_path.clone())?,
        None => Config::try_new()?,
    };
    config.set_debug(cli.debug);
    match cli.command {
        Commands::Utils { subcommand } => subcommand.invoke(&mut config),
        Commands::Music { subcommand } => subcommand.invoke(&mut config),
        Commands::Config { subcommand } => subcommand.invoke(&mut config),
        Commands::Gh { subcommand } => subcommand.invoke(&mut config),
        Commands::Movies { subcommand } => subcommand.invoke(&mut config),
        Commands::Sync { subcommand } => subcommand.invoke(&mut config),
        Commands::Completions => Commands::gen_completions(&mut config),
        Commands::Man => Commands::gen_man(&mut config),
    }
}
