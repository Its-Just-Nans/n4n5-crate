//! To see all subcommands, run:
//! ```shell
//! n4n5 settings
//! ```
//!
use std::{
    fs::write,
    fs::{create_dir_all, read_to_string},
    path::PathBuf,
    process::Command,
};

use clap::{arg, ArgMatches, Command as ClapCommand};
use serde::{Deserialize, Serialize};
use std::io::Write;

use crate::{
    cli::{input_path, CliCommand},
    config::Config,
    config_path, config_sub_path,
};

use super::{config, movies::Movies};

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
    pub path_vscode_extenstions: Option<String>,
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
            .subcommand(ClapCommand::new("all").about("sync all"))
            .arg_required_else_help(true)
    }

    fn invoke(config: &mut Config, args_matches: &ArgMatches) {
        if let Some(matches) = args_matches.subcommand_matches("settings") {
            if let Some(matches) = matches.subcommand_matches("add") {
                SyncCliCommand::add_file(config, matches);
            } else {
                SyncCliCommand::save_files(config, matches);
            }
        } else if let Some(matches) = args_matches.subcommand_matches("movies") {
            Movies::sync_movies(config, Some(matches));
        } else if let Some(_matches) = args_matches.subcommand_matches("programs") {
            SyncCliCommand::sync_programs(config);
        } else if let Some(_matches) = args_matches.subcommand_matches("all") {
            SyncCliCommand::sync_all(config, args_matches);
        }
    }
}

impl SyncCliCommand {
    /// Get the home path
    fn get_home_path() -> (PathBuf, String) {
        let path_buf = home::home_dir().expect("Unable to get home directory");
        let path = path_buf.clone();
        let path = path.to_str().expect("Unable to convert path to string");
        (path_buf, path.to_string())
    }

    /// Save the files to the specified folder
    fn save_files(config: &mut Config, _matches: &ArgMatches) {
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

        let (home_pathbuf, _) = SyncCliCommand::get_home_path();
        let files_path = files_path
            .iter()
            .map(
                |path| match PathBuf::from(path).strip_prefix(&home_pathbuf) {
                    Ok(path) => path.to_path_buf(),
                    Err(_) => PathBuf::from(path),
                },
            )
            .collect::<Vec<PathBuf>>();
        println!("Saving {} files to {:?}", files_path.len(), folder_path);
        for file_path_to_save in files_path {
            let input_path = home_pathbuf.join(&file_path_to_save);
            let file_content = read_to_string(&input_path)
                .unwrap_or_else(|_| panic!("Unable to open {:?}", input_path));
            let save_path = folder_path.join(&file_path_to_save);
            println!("Saving {:?} to {:?}", input_path, save_path);
            if let Some(parent) = save_path.parent() {
                create_dir_all(parent)
                    .unwrap_or_else(|_| panic!("Unable to create save folder {:?}", parent));
            }
            let mut file = std::fs::File::create(&save_path)
                .unwrap_or_else(|_| panic!("Unable to create file {:?}", save_path));
            file.write_all(file_content.as_bytes())
                .expect("Unable to write to file");
        }
    }

    /// Add a file to the list of files to save
    fn add_file(config: &mut Config, _matches: &ArgMatches) {
        println!("Please enter the path to the file to add:");
        let file_path = input_path();
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
        });
    }

    /// Sync the cargo programs
    fn sync_programs_cargo(config: &mut Config) {
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
            .output()
            .expect("Unable to run cargo");
        let cargo_programs = String::from_utf8_lossy(&cargo_programs.stdout).to_string();
        write(&cargo_path, cargo_programs).unwrap_or_else(|_| {
            panic!("Unable to write cargo programs to {}", cargo_path.display())
        });
        println!("Saved cargo programs to {}", cargo_path.display());
    }

    /// Sync the nix-env programs
    fn sync_programs_nix(config: &mut Config) {
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
            .output()
            .expect("Unable to run nix");
        let nix_programs = String::from_utf8_lossy(&nix_programs.stdout).to_string();
        write(&nix_path, nix_programs)
            .unwrap_or_else(|_| panic!("Unable to write nix programs to {}", nix_path.display()));
        println!("Saved nix programs to {}", nix_path.display());
    }

    /// Sync the vscode extensions
    fn sync_programs_vscode(config: &mut Config) {
        let vscode_path = config_sub_path!(
            config,
            sync,
            SyncCliCommand,
            programs,
            ProgramsConfig,
            path_vscode_extenstions,
            "vscode extensions"
        );
        let vscode_extensions = Command::new("sh")
            .arg("-c")
            .arg("code --list-extensions")
            .output()
            .expect("Unable to run vscode");
        let vscode_extensions = String::from_utf8_lossy(&vscode_extensions.stdout).to_string();
        write(&vscode_path, vscode_extensions).unwrap_or_else(|_| {
            panic!(
                "Unable to write vscode extensions to {}",
                vscode_path.display()
            )
        });
        println!("Saved vscode extensions to {}", vscode_path.display());
    }

    /// Sync the programs
    fn sync_programs(config: &mut Config) {
        println!("Syncing programs");
        SyncCliCommand::sync_programs_cargo(config);
        SyncCliCommand::sync_programs_nix(config);
        SyncCliCommand::sync_programs_vscode(config);
    }

    /// Sync all
    fn sync_all(config: &mut Config, args_matches: &ArgMatches) {
        Movies::sync_movies(config, None);
        println!();
        SyncCliCommand::save_files(config, args_matches);
        println!();
        SyncCliCommand::sync_programs(config);
    }
}
