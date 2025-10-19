//! To see all subcommands, run:
//! ```shell
//! n4n5 music
//! ```
//!

use std::{path::PathBuf, process::Command};

use clap::{ArgAction, Subcommand};
use music_exporter::PlatformType;
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;

use crate::{cli::input_path, config::Config, config_path, errors::GeneralError};

/// Movies configuration
#[derive(Deserialize, Serialize, Default)]
pub struct MusicCliCommand {
    /// Path to the movies file
    pub music_file: Option<String>,

    /// env path
    pub env_path: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum MusicSubcommand {
    /// Save music
    Sync,

    /// Open music file
    Open {
        /// Print the path only
        #[arg(short = 'p', long = "path", action = ArgAction::SetTrue)]
        show_path_only: bool,
    },
}

impl MusicSubcommand {
    /// invoke the subcommand
    /// # Errors
    /// Error if error in subcommand
    pub fn invoke(self, config: &mut Config) -> Result<(), GeneralError> {
        match self {
            MusicSubcommand::Sync => MusicCliCommand::sync_music(config),
            MusicSubcommand::Open { show_path_only } => {
                MusicCliCommand::open_music_file(config, show_path_only)
            }
        }
    }
}

impl MusicCliCommand {
    /// Get the music file path
    /// # Errors
    /// Fails if the file cannot be found
    pub fn get_music_file_path(config: &mut Config) -> Result<PathBuf, GeneralError> {
        let path = config_path!(
            config,
            music,
            MusicCliCommand,
            music_file,
            "the file for music"
        );
        Ok(path)
    }

    /// open music file
    /// # Errors
    /// Fails if the file cannot be opened
    pub fn open_music_file(config: &mut Config, print_path: bool) -> Result<(), GeneralError> {
        let music_file = MusicCliCommand::get_music_file_path(config)?;
        if print_path {
            println!("{}", music_file.display());
            return Ok(());
        }
        println!("Opening music file at {}", music_file.display());
        Command::new("vi").arg(&music_file).spawn()?.wait()?;
        Ok(())
    }

    /// Sync music
    /// # Errors
    /// Fails if the music file cannot be found
    pub fn sync_music(config: &mut Config) -> Result<(), GeneralError> {
        let rt = Runtime::new()?;

        let music_file = MusicCliCommand::get_music_file_path(config)?;
        let env_path = config_path!(config, music, MusicCliCommand, env_path, "the env path");

        println!("music file: '{}'", music_file.display());
        let platforms = vec![
            PlatformType::Deezer,
            PlatformType::Spotify,
            PlatformType::Youtube,
        ];

        rt.block_on(async {
            env_logger::builder()
                .filter_level(log::LevelFilter::Info)
                .format_target(false)
                .format_timestamp(None)
                .init();
            music_exporter::cli_main(music_file, Some(env_path), &platforms).await
        });
        Ok(())
    }
}
