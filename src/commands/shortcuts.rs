//! Shortcuts related subcommands

use clap::Subcommand;

use crate::errors::GeneralError;

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
                use crate::commands::Commands;
                use git_mover::PlatformType;

                let git_mover_inst = git_mover::GitMoverCli {
                    source: Some(PlatformType::Github),
                    destination: Some(PlatformType::Codeberg),
                    manual: true,
                    ..Default::default()
                };
                Commands::git_mover(git_mover_inst)
            }
        }
    }
}
