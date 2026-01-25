//! To see all subcommands, run:
//!
//! ```shell
//! n4n5 utils
//! ```
//!
use clap::Subcommand;

use crate::{
    commands::utils::{list_crates::UtilsListCrates, music::MusicSubcommand},
    config::Config,
    errors::GeneralError,
};

/// Movies configuration
#[derive(Subcommand, Debug)]
pub enum UtilsSubCommand {
    /// [`list_crates`] subcommand
    #[command(name = "list_crates")]
    ListCrates(UtilsListCrates),

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
        subcommand: MusicSubcommand,
    },
}

impl UtilsSubCommand {
    /// invoke the subcommand
    /// # Errors
    /// Error if error in subcommand
    pub fn invoke(self, config: &mut Config) -> Result<(), GeneralError> {
        match self {
            UtilsSubCommand::ListCrates(subcommand) => subcommand.list_crates(config),
            #[cfg(feature = "pngtools")]
            UtilsSubCommand::PngTools => Self::pngtools(),
            #[cfg(feature = "git-mover")]
            UtilsSubCommand::GitMover(git_mover) => Self::git_mover(git_mover),
            #[cfg(feature = "galion")]
            UtilsSubCommand::Galion(galion_args) => Self::galion(galion_args),
            UtilsSubCommand::Music { subcommand } => subcommand.invoke(config),
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
