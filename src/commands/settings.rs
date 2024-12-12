use std::{
    fs::{create_dir_all, read_to_string},
    path::PathBuf,
};

use clap::{ArgMatches, Command};
use serde::{Deserialize, Serialize};
use std::io::Write;

use crate::{
    cli::{input_path, CliCommand},
    config::Config,
};

/// Movies configuration
#[derive(Deserialize, Serialize, Default)]
pub struct Settings {
    /// Path to the movies file
    pub file_paths: Vec<String>,

    /// Path to the folder where to save the files
    pub save_folder_path: Option<String>,
}

impl CliCommand for Settings {
    fn get_subcommand() -> clap::Command {
        Command::new("settings")
            .about("settings subcommand")
            .subcommand(Command::new("add").about("add a file to save"))
            .subcommand(Command::new("save").about("save settings"))
            .arg_required_else_help(true)
    }

    fn invoke(config: &mut Config, args_matches: &ArgMatches) {
        if let Some(matches) = args_matches.subcommand_matches("save") {
            Settings::save_files(config, matches);
        } else if let Some(matches) = args_matches.subcommand_matches("add") {
            Settings::add_file(config, matches);
        }
    }
}

impl Settings {
    /// Get the home path
    fn get_home_path() -> (PathBuf, String) {
        let path_buf = home::home_dir().expect("Unable to get home directory");
        let path = path_buf.clone();
        let path = path.to_str().expect("Unable to convert path to string");
        (path_buf, path.to_string())
    }

    /// Save the files to the specified folder
    fn save_files(config: &mut Config, _matches: &ArgMatches) {
        let files_path = match &config.config_data.settings {
            Some(settings) => settings.file_paths.clone(),
            None => Vec::new(),
        };
        let folder_path = match &config.config_data.settings {
            Some(settings) => settings.save_folder_path.clone(),
            None => None,
        };
        let folder_path = match folder_path {
            Some(folder_path) => PathBuf::from(folder_path),
            None => {
                println!("Please enter the path to the folder where to save the files:");
                let file_path = input_path();
                let cloned_path = file_path.1.clone();
                config.update(|config_data| {
                    if let Some(settings) = config_data.settings.as_mut() {
                        settings.save_folder_path = Some(cloned_path);
                    } else {
                        config_data.settings = Some(Settings {
                            file_paths: vec![],
                            save_folder_path: Some(cloned_path),
                        });
                    }
                    config_data
                });
                file_path.0
            }
        };
        let (home_pathbuf, _) = Settings::get_home_path();
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
                create_dir_all(parent).expect("Unable to create save folder");
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
            if let Some(settings) = config_data.settings.as_mut() {
                settings.file_paths.push(cloned_path);
            } else {
                config_data.settings = Some(Settings {
                    file_paths: vec![cloned_path],
                    save_folder_path: None,
                });
            }
            config_data
        });
    }
}
