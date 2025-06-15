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

use clap::{arg, ArgMatches, Command as ClapCommand};
use serde::{Deserialize, Serialize};

use crate::{
    cli::{input_no, input_path, CliCommand},
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

impl CliCommand for SyncCliCommand {
    fn get_subcommand() -> ClapCommand {
        ClapCommand::new("sync")
            .about("sync subcommand")
            .subcommand(
                ClapCommand::new("settings")
                    .about("save settings")
                    .subcommand(ClapCommand::new("add").about("add a file to save"))
                    .subcommand(ClapCommand::new("all").about("add a file to save"))
                    .arg_required_else_help(true),
            )
            .subcommand(
                ClapCommand::new("movies").about("sync movies").arg(
                    arg!(
                        -j --json ... "print as json"
                    )
                    .required(false),
                ),
            )
            .subcommand(ClapCommand::new("programs").about("sync programs"))
            .subcommand(ClapCommand::new("music").about("sync music"))
            .subcommand(ClapCommand::new("all").about("sync all"))
            .arg_required_else_help(true)
    }

    fn invoke(config: &mut Config, args_matches: &ArgMatches) -> Result<(), GeneralError> {
        if let Some(matches) = args_matches.subcommand_matches("settings") {
            if let Some(matches) = matches.subcommand_matches("add") {
                return SyncCliCommand::add_file(config, matches);
            } else if let Some(_matches) = matches.subcommand_matches("all") {
                return SyncCliCommand::save_files(config);
            }
        } else if let Some(matches) = args_matches.subcommand_matches("movies") {
            return Movies::sync_movies(config, Some(matches));
        } else if let Some(_matches) = args_matches.subcommand_matches("programs") {
            return SyncCliCommand::sync_programs(config);
        } else if let Some(matches) = args_matches.subcommand_matches("all") {
            return SyncCliCommand::sync_all(config, matches);
        } else if let Some(matches) = args_matches.subcommand_matches("music") {
            return MusicCliCommand::sync_music(config, matches);
        }
        Ok(())
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
            let mut file_input = File::open(&input_path)?;
            let save_path = folder_path.join(&file_path_to_save);
            println!("- '{}' to '{}'", input_path.display(), save_path.display());
            if let Some(parent) = save_path.parent() {
                create_dir_all(parent)?;
            }
            let mut file = File::create(&save_path)?;

            io::copy(&mut file_input, &mut file)?;
        }
        Ok(())
    }

    /// Add a file to the list of files to save
    /// # Errors
    /// Returns an error if the file path is invalid
    fn add_file(config: &mut Config, _matches: &ArgMatches) -> Result<(), GeneralError> {
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
    fn sync_all(config: &mut Config, _matches: &ArgMatches) -> Result<(), GeneralError> {
        config.use_input = false;
        if config.debug > 1 {
            println!("Syncing all");
        }
        if config.config_data.movies.is_some() {
            Movies::sync_movies(config, None)?;
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
            Gh::sync_github(config, None)?;
        }
        Ok(())
    }
}
