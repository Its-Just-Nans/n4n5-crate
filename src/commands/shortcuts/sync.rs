//! Shortucts Sync

use std::thread;

use crate::commands::gh::lib::Gh;
use crate::commands::movies::Movies;
use crate::commands::shortcuts::ShortcutsSubcommand;
use crate::config::Config;
use crate::errors::GeneralError;

impl ShortcutsSubcommand {
    /// Sync all
    /// # Errors
    /// Returns an error if any of the subcommands fails
    pub(crate) fn sync_all(config: &mut Config) -> Result<(), GeneralError> {
        config.use_input = false;
        if config.debug > 1 {
            println!("Syncing all");
        }

        if config.config_data.movies.is_some() {
            Movies::pre_sync_movies(config)?;
        }
        // if config.config_data.sync.is_some() {
        // SyncCliCommand::pre_save_files(config)?;
        // SyncCliCommand::pre_sync_programs(config)?;
        // }
        if config.config_data.gh.is_some() {
            Gh::pre_sync_github(config)?;
        }

        // real sync
        thread::scope(|s| {
            if config.config_data.movies.is_some() {
                s.spawn(|| Movies::sync_movies(config, false));
            }
            // if config.config_data.sync.is_some() {
            // s.spawn(|| SyncCliCommand::save_files(config));
            // s.spawn(|| SyncCliCommand::sync_programs(config));
            // }
            if config.config_data.gh.is_some() {
                s.spawn(|| Gh::save_pulls(config));
                s.spawn(|| Gh::save_projects(config, false));
            }
        });
        Ok(())
    }
}
