//! To see all subcommands, run:
//! ```shell
//! n4n5 music
//! ```
//!use serde::Serialize;

use std::path::PathBuf;

use clap::{ArgMatches, Command as ClapCommand};
use music_exporter::PlatformType;
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;

use crate::{
    cli::{input_path, CliCommand},
    config::Config,
    config_path,
};

/// Movies configuration
#[derive(Deserialize, Serialize, Default)]
pub struct MusicCliCommand {
    /// Path to the movies file
    pub music_file: Option<String>,

    /// env path
    pub env_path: Option<String>,
}

impl CliCommand for MusicCliCommand {
    fn get_subcommand() -> ClapCommand {
        ClapCommand::new("music")
            .about("sync subcommand")
            .subcommand(ClapCommand::new("sync").about("save music"))
            .arg_required_else_help(true)
    }
    fn invoke(config: &mut Config, args_matches: &ArgMatches) {
        if let Some(matches) = args_matches.subcommand_matches("sync") {
            MusicCliCommand::sync_music(config, matches);
        }
    }
}

impl MusicCliCommand {
    /// Get the music file path
    pub fn get_music_file_path(config: &mut Config) -> PathBuf {
        config_path!(
            config,
            music,
            MusicCliCommand,
            music_file,
            "the file for music"
        )
    }

    /// Sync music
    pub fn sync_music(config: &mut Config, _matches: &ArgMatches) {
        let rt = Runtime::new().expect("Failed to create tokio runtime");

        let music_file = MusicCliCommand::get_music_file_path(config);
        let env_path = config_path!(config, music, MusicCliCommand, env_path, "the env path");

        println!("music file: {:?}", music_file);
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
    }
}
