//! To see all subcommands, run:
//! ```shell
//! n4n5 sync
//! ```
//!
use std::{
    fs::{create_dir_all, write, File},
    io,
    path::PathBuf,
    process::Command,
};

use clap::{arg, Parser, Subcommand};
use serde::{Deserialize, Serialize};

use crate::{
    cli::{input_no, input_path},
    commands::gh::lib::Gh,
    config::Config,
    config_path, config_sub_path,
    errors::GeneralError,
};

use super::{movies::Movies, music::MusicCliCommand};

/// Movies configuration
#[derive(Deserialize, Serialize, Default)]
pub struct SyncCliCommand {
    /// Path to the movies file
    pub file_paths: Vec<String>,

    /// Path to the folder where to save the files
    pub save_folder_path: Option<String>,

    /// Programs configuration
    pub programs: Option<ProgramsConfig>,
}

/// Programs configuration
#[derive(Deserialize, Serialize, Default)]
pub struct ProgramsConfig {
    /// Path to the cargo programs
    pub path_cargo_programs: Option<String>,
    /// Path to the vscode extensions
    pub path_vscode_extensions: Option<String>,
    /// Path to the nix programs
    pub path_nix: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum SyncSubcommand {
    /// save settings
    Settings(SettingsCommand),

    /// sync movies
    Movies {
        /// print as json
        #[arg(short = 'j', long = "json")]
        print_json: bool,
    },

    /// sync programs
    Programs,

    /// sync music
    Music,

    /// sync all
    All,
}

/// settings subcommand
#[derive(Parser, Debug)]
#[command(arg_required_else_help = true)]
pub struct SettingsCommand {
    /// settings subcommand
    #[command(subcommand)]
    pub action: SettingsAction,
}

#[derive(Subcommand, Debug)]
pub enum SettingsAction {
    /// add a file to save
    Add,

    /// add all files to save
    All,
}

impl SyncSubcommand {
    /// invoke subcommand
    /// # Errors
    /// Error if error in subcommand
    pub fn invoke(self, config: &mut Config) -> Result<(), GeneralError> {
        match self {
            Self::All => SyncCliCommand::sync_all(config),
            Self::Music => MusicCliCommand::sync_music(config),
            Self::Movies { print_json } => Movies::sync_movies(config, print_json),
            Self::Programs => SyncCliCommand::sync_programs(config),
            Self::Settings(settings) => match settings.action {
                SettingsAction::Add => SyncCliCommand::add_file(config),
                SettingsAction::All => SyncCliCommand::save_files(config),
            },
        }
    }
}

impl SyncCliCommand {
    /// Get the home path
    /// # Errors
    /// Returns an error if the home directory cannot be found
    fn get_home_path() -> Result<(PathBuf, String), GeneralError> {
        let path_buf = home::home_dir()
            .ok_or_else(|| GeneralError::new("Cannot find home directory".to_string()))?;
        let path = path_buf.clone();
        let path = path.to_str().ok_or_else(|| {
            GeneralError::new("Cannot convert home directory to string".to_string())
        })?;
        Ok((path_buf, path.to_string()))
    }

    /// Save the files to the specified folder
    /// # Errors
    /// Returns an error if the file cannot be saved
    fn save_files(config: &mut Config) -> Result<(), GeneralError> {
        let files_path = match &config.config_data.sync {
            Some(settings) => settings.file_paths.clone(),
            None => Vec::new(),
        };
        let folder_path = config_path!(
            config,
            sync,
            SyncCliCommand,
            save_folder_path,
            "settings folder"
        );

        let (home_pathbuf, _) = SyncCliCommand::get_home_path()?;
        let files_path = files_path
            .iter()
            .map(
                |path| match PathBuf::from(path).strip_prefix(&home_pathbuf) {
                    Ok(path) => path.to_path_buf(),
                    Err(_) => PathBuf::from(path),
                },
            )
            .collect::<Vec<PathBuf>>();
        println!(
            "Saving {} files to '{}'",
            files_path.len(),
            folder_path.display()
        );
        for file_path_to_save in files_path {
            let input_path = home_pathbuf.join(&file_path_to_save);
            let mut file_input =
                File::open(&input_path).map_err(|e| format!("{e} for {}", input_path.display()))?;
            let save_path = folder_path.join(&file_path_to_save);
            println!("- '{}' to '{}'", input_path.display(), save_path.display());
            if let Some(parent) = save_path.parent() {
                create_dir_all(parent).map_err(|e| format!("{e} for {}", parent.display()))?;
            }
            let mut file = File::create(&save_path)?;

            io::copy(&mut file_input, &mut file)?;
        }
        Ok(())
    }

    /// Add a file to the list of files to save
    /// # Errors
    /// Returns an error if the file path is invalid
    fn add_file(config: &mut Config) -> Result<(), GeneralError> {
        println!("Please enter the path to the file to add:");
        let file_path = input_path()?;
        let cloned_path = file_path.1.clone();
        config.update(|config_data| {
            if let Some(local_config) = config_data.sync.as_mut() {
                local_config.file_paths.push(cloned_path);
            } else {
                config_data.sync = Some(SyncCliCommand {
                    file_paths: vec![cloned_path],
                    ..Default::default()
                });
            }
            config_data
        })?;
        Ok(())
    }

    /// Sync the cargo programs
    /// # Errors
    /// Returns an error if the command fails
    fn sync_programs_cargo(config: &mut Config) -> Result<(), GeneralError> {
        if !config.use_input {
            return Ok(());
        }
        if let Some(sync) = &config.config_data.sync {
            if let Some(programs) = &sync.programs {
                if programs.path_cargo_programs.is_none() {
                    println!("No cargo programs path found, do you want to add one? (y/n)");
                    if input_no() {
                        return Ok(());
                    }
                }
            }
        }
        let cargo_path = config_sub_path!(
            config,
            sync,
            SyncCliCommand,
            programs,
            ProgramsConfig,
            path_cargo_programs,
            "cargo programs"
        );
        let cargo_programs = Command::new("sh")
            .arg("-c")
            .arg("cargo install --list | grep -v ':$' | sed 's/^ *//'")
            .output()?;
        let cargo_programs = String::from_utf8_lossy(&cargo_programs.stdout).to_string();
        write(&cargo_path, cargo_programs)?;
        println!("Saved cargo programs to {}", cargo_path.display());
        Ok(())
    }

    /// Sync the nix-env programs
    /// # Errors
    /// Returns an error if the command fails
    fn sync_programs_nix(config: &mut Config) -> Result<(), GeneralError> {
        if !config.use_input {
            return Ok(());
        }
        if let Some(sync) = &config.config_data.sync {
            if let Some(programs) = &sync.programs {
                if programs.path_nix.is_none() {
                    println!("No nix programs path found, do you want to add one? (y/n)");
                    if input_no() {
                        return Ok(());
                    }
                }
            }
        }
        let nix_path = config_sub_path!(
            config,
            sync,
            SyncCliCommand,
            programs,
            ProgramsConfig,
            path_nix,
            "nix programs"
        );
        let nix_programs = Command::new("sh")
            .arg("-c")
            .arg("nix-env --query | cut -d'-' -f 1")
            .output()?;
        let nix_programs = String::from_utf8_lossy(&nix_programs.stdout).to_string();
        write(&nix_path, nix_programs)?;
        println!("Saved nix programs to {}", nix_path.display());
        Ok(())
    }

    /// Sync the vscode extensions
    /// # Errors
    /// Returns an error if the command fails
    fn sync_programs_vscode(config: &mut Config) -> Result<(), GeneralError> {
        if !config.use_input {
            return Ok(());
        }
        if let Some(sync) = &config.config_data.sync {
            if let Some(programs) = &sync.programs {
                if programs.path_vscode_extensions.is_none() {
                    println!("No vscode extensions path found, do you want to add one? (y/n)");
                    if input_no() {
                        return Ok(());
                    }
                }
            }
        }
        let vscode_path = config_sub_path!(
            config,
            sync,
            SyncCliCommand,
            programs,
            ProgramsConfig,
            path_vscode_extensions,
            "vscode extensions"
        );
        let vscode_extensions = Command::new("sh")
            .arg("-c")
            .arg("code --list-extensions")
            .output()?;
        let vscode_extensions = String::from_utf8_lossy(&vscode_extensions.stdout).to_string();
        write(&vscode_path, vscode_extensions)?;
        println!("Saved vscode extensions to {}", vscode_path.display());
        Ok(())
    }

    /// Sync the programs
    /// # Errors
    /// Returns an error if any of the subcommands fails
    fn sync_programs(config: &mut Config) -> Result<(), GeneralError> {
        println!("Syncing programs");
        SyncCliCommand::sync_programs_cargo(config)?;
        SyncCliCommand::sync_programs_nix(config)?;
        SyncCliCommand::sync_programs_vscode(config)?;
        Ok(())
    }

    /// Sync all
    /// # Errors
    /// Returns an error if any of the subcommands fails
    fn sync_all(config: &mut Config) -> Result<(), GeneralError> {
        config.use_input = false;
        if config.debug > 1 {
            println!("Syncing all");
        }
        if config.config_data.movies.is_some() {
            Movies::sync_movies(config, false)?;
            println!();
        }
        if config.config_data.sync.is_some() {
            SyncCliCommand::save_files(config)?;
            println!();
        }
        if config.config_data.sync.is_some() {
            SyncCliCommand::sync_programs(config)?;
            println!();
        }
        if config.config_data.gh.is_some() {
            Gh::sync_github(config, false)?;
        }
        Ok(())
    }
}
