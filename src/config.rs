//! Configuration module

use crate::{
    commands::{gh::lib::Gh, movies::Movies, music::MusicCliCommand, sync::SyncCliCommand},
    errors::GeneralError,
};
use home::home_dir;
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, create_dir_all, read_to_string},
    io::Write,
    path::PathBuf,
    str,
};

/// Configuration object
/// It's linked to a configuration file
#[derive(Deserialize, Default)]
pub struct Config {
    /// debug level
    pub debug: u8,

    /// whether to use input for configuration
    pub use_input: bool,

    /// path to the configuration file
    pub config_path: PathBuf,

    /// actual configuration data
    pub config_data: ConfigData,
}

/// Configuration
/// Configuration data is stored in a TOML file
/// The configuration is separated into different sections
#[derive(Deserialize, Serialize, Default)]
pub struct ConfigData {
    /// Movies configuration
    pub movies: Option<Movies>,

    /// Sync configuration
    pub sync: Option<SyncCliCommand>,

    /// Github configuration
    pub gh: Option<Gh>,

    /// Music configuration
    pub music: Option<MusicCliCommand>,
}

impl Config {
    /// Parse the config file
    /// # Errors
    /// Error if toml parse fails
    fn parse_config(str_config: &str, path_config: PathBuf) -> Result<Self, GeneralError> {
        let config_data = toml::from_str(str_config)?;
        Ok(Config {
            debug: 0,
            use_input: true,
            config_path: path_config,
            config_data,
        })
    }

    /// Create a new Config object from the default path
    /// # Errors
    /// Error if the file can't be opened
    pub fn try_new() -> Result<Self, GeneralError> {
        let config_path = Config::get_config_path()?;
        Self::try_new_from_path(config_path)
    }

    /// Save the config data to the config file
    /// # Errors
    /// Returns an error if the file can't be written to
    pub fn save(&self) -> Result<(), GeneralError> {
        let config_str = toml::to_string(&self.config_data)?;
        let mut file = File::create(&self.config_path)?;
        file.write_all(config_str.as_bytes())?;
        Ok(())
    }

    /// Create a new Config object from a custom path
    /// # Errors
    /// Returns an error if the file can't be opened
    pub fn try_new_from_path(custom_path: PathBuf) -> Result<Self, GeneralError> {
        let contents = read_to_string(custom_path.clone())
            .map_err(|e| format!("Unable to open '{}': {e}", custom_path.display()))?;
        Config::parse_config(&contents, custom_path)
    }

    /// Set the debug value
    pub fn set_debug(&mut self, value: u8) {
        self.debug = value;
    }

    /// Get the path to the config file
    /// # Errors
    /// Error if the home directory can't be found
    pub fn get_config_path() -> Result<PathBuf, GeneralError> {
        let home_dir = match home_dir() {
            Some(path) if !path.as_os_str().is_empty() => path,
            _ => {
                return Err(GeneralError::new(
                    "Unable to get your home dir! home::home_dir() isn't working",
                ));
            }
        };
        let config_directory = home_dir.join(".config").join(".n4n5");
        let config_path = config_directory.join("config.toml");
        create_dir_all(config_directory).map_err(|e| format!("Unable to create config dir {e}"))?;
        if !config_path.exists() {
            let mut file = File::create(&config_path)
                .map_err(|e| format!("Unable to create config file {e}"))?;
            file.write_all(b"")
                .map_err(|e| format!("Unable to write to config file: {e}"))?;
        }
        Ok(config_path)
    }

    /// Update the config data and save it to the config file
    /// # Errors
    /// Returns an error if the file can't be written to
    pub fn update(
        &mut self,
        updater_fn: impl FnOnce(&mut ConfigData) -> &mut ConfigData,
    ) -> Result<(), GeneralError> {
        updater_fn(&mut self.config_data);
        self.save()?;
        Ok(())
    }
}
