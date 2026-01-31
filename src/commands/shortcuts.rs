//! Shortcuts related subcommands

use clap::Subcommand;

use crate::{commands::utils::lib::UtilsSubCommand, errors::GeneralError};

/// Shortcuts related subcommands
#[derive(Subcommand, Debug, Clone)]
pub(crate) enum ShortcutsSubcommand {
    /// Sync git repos between GitHub and Codeberg
    #[cfg(feature = "git-mover")]
    SyncGit,
}

impl ShortcutsSubcommand {
    /// Run the selected shortcut subcommand
    ///
    /// # Errors
    ///
    /// Returns `GeneralError` if an error occurs during execution
    pub fn run(&self) -> Result<(), GeneralError> {
        match self {
            #[cfg(feature = "git-mover")]
            ShortcutsSubcommand::SyncGit => {
                use git_mover::PlatformType;
                let git_mover_inst = git_mover::GitMoverCli {
                    source: Some(PlatformType::Github),
                    destination: Some(PlatformType::Codeberg),
                    manual: true,
                    ..Default::default()
                };
                UtilsSubCommand::git_mover(git_mover_inst)
            }
        }
    }
}
