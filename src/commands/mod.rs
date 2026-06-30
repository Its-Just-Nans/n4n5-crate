//! This module contains all the commands that can be executed.

use clap::{CommandFactory, Subcommand};
use clap_complete::{
    generate_to,
    shells::{Bash, Elvish, Fish, PowerShell, Zsh},
};
use home::home_dir;
use std::fs::create_dir_all;

use crate::{cli::CliArgs, commands::list_crates::ListCrates};
use crate::{
    commands::{
        gh::lib::GhSubCommand, movies::MoviesSubCommand, shortcuts::ShortcutsSubcommand,
        sync::SyncSubcommand,
    },
    config::Config,
    errors::GeneralError,
};

use crate::commands::config::ConfigSubcommand;

pub(crate) mod config;
pub(crate) mod gh;
pub(crate) mod list_crates;
pub(crate) mod man;
pub(crate) mod movies;
pub(crate) mod music;
pub(crate) mod share;
pub(crate) mod shortcuts;
pub(crate) mod sync;
pub(crate) mod watching;

/// Main commands enum
#[derive(Subcommand, Debug)]
pub(crate) enum Commands {
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

    /// list crates subcommand
    #[command(name = "list_crates")]
    ListCrates(ListCrates),

    /// Launch pngtools cli
    #[cfg(feature = "pngtools")]
    #[command(name = "pngtools")]
    PngTools,

    /// Launch git-mover cli
    #[cfg(feature = "git-mover")]
    #[command(name = "git-mover")]
    GitMover(git_mover::GitMoverCli),

    /// Launch galion tui
    #[cfg(feature = "galion")]
    #[command(name = "galion")]
    Galion(galion::GalionArgs),

    /// music subcommand
    Music {
        /// list of subcommands
        #[command(subcommand)]
        subcommand: crate::commands::music::MusicSubcommand,
    },

    /// Quick http server share
    Share,

    /// List watching repos
    Watching,
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

    /// Invoke subcommands
    /// # Errors
    /// Fails if subcommand fails
    pub(crate) fn invoke(self, config: &mut Config) -> Result<(), GeneralError> {
        match self {
            Commands::Config { subcommand } => subcommand.invoke(config),
            Commands::Gh { subcommand } => subcommand.invoke(config),
            Commands::Movies { subcommand } => subcommand.invoke(config),
            Commands::Sync { subcommand } => subcommand.invoke(config),
            Commands::Completions => Commands::gen_completions(config),
            Commands::Man => Commands::gen_man(config),
            Commands::Shortcuts { subcommand } => subcommand.run(),
            Commands::ListCrates(subcommand) => subcommand.list_crates(config),

            #[cfg(feature = "pngtools")]
            Commands::PngTools => Self::pngtools(),
            #[cfg(feature = "git-mover")]
            Commands::GitMover(git_mover) => Self::git_mover(git_mover),
            #[cfg(feature = "galion")]
            Commands::Galion(galion_args) => Self::galion(galion_args),
            Commands::Music { subcommand } => subcommand.invoke(config),
            Commands::Share => Self::share(),
            Commands::Watching => Self::watching(),
        }
    }

    /// Run pngtools cli
    /// # Errors
    /// Fails if pngtools fails
    #[cfg(feature = "pngtools")]
    pub fn pngtools() -> Result<(), GeneralError> {
        pngtools::run_cli().map_err(|e| ("Error with pngtools", e))?;
        Ok(())
    }

    /// Share main func
    /// # Errors
    /// Return error if the sharing server is failing
    pub(crate) fn share() -> Result<(), GeneralError> {
        use crate::commands::share::main;

        use tokio::runtime::Runtime;

        let rt = Runtime::new()?;
        rt.block_on(async {
            env_logger::builder()
                .filter_level(log::LevelFilter::Info)
                .format_target(false)
                .format_timestamp(None)
                .init();
            main()
                .await
                .map_err(|e| GeneralError::new_with_source("Error from sharing", e))
        })?;
        Ok(())
    }

    /// Run git-mover cli
    /// # Errors
    /// Fails if git-mover fails
    #[cfg(feature = "git-mover")]
    pub fn git_mover(git_mover_inst: git_mover::GitMoverCli) -> Result<(), GeneralError> {
        use tokio::runtime::Runtime;

        let rt = Runtime::new()?;
        rt.block_on(async {
            env_logger::builder()
                .filter_level(log::LevelFilter::Info)
                .format_target(false)
                .format_timestamp(None)
                .init();
            git_mover_inst
                .main()
                .await
                .map_err(|e| GeneralError::new_with_source("Error with git-mover", e))
        })?;
        Ok(())
    }

    /// Run galion cli
    /// # Errors
    /// Fails if galion fails
    #[cfg(feature = "galion")]
    pub fn galion(galion_args: galion::GalionArgs) -> Result<(), GeneralError> {
        use galion::GalionApp;

        let app = GalionApp::try_from_galion_args(galion_args)
            .map_err(|e| GeneralError::new_with_source("Error with galion", e))?;
        app.run_tui()
            .map_err(|e| GeneralError::new_with_source("Error with galion", e))?;
        Ok(())
    }
}
