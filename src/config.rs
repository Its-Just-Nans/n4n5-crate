//! Configuration module

use crate::{
    commands::{gh::lib::Gh, movies::Movies, sync::SyncCliCommand, utils::music::MusicCliCommand},
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
pub struct Config {
    /// path to the configuration file
    pub config_path: PathBuf,

    /// actual configuration data
    pub config_data: ConfigData,

    /// Turn debugging information on
    pub debug: u8,
    /// whether to use input for configuration
    pub use_input: bool,
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
    /// Create a new Config object from the default path
    /// # Errors
    /// Error if the file can't be opened
    pub fn try_new(
        config_path: Option<PathBuf>,
        debug: u8,
        use_input: bool,
    ) -> Result<Self, GeneralError> {
        let config_path = match config_path {
            Some(p) => p,
            None => Config::get_config_path()?,
        };
        let contents = read_to_string(&config_path)
            .map_err(|e| (format!("Unable to open '{}'", config_path.display()), e))?;
        let config_data = toml::from_str(&contents)?;
        Ok(Config {
            config_path,
            config_data,
            debug,
            use_input,
        })
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
            let mut file =
                File::create(&config_path).map_err(|e| ("Unable to create config file", e))?;
            file.write_all(b"")
                .map_err(|e| ("Unable to write to config file", e))?;
        }
        Ok(config_path)
    }

    /// Update the config data and save it to the config file
    /// # Errors
    /// Returns an error if the file can't be written to
    pub fn update(&mut self, updater_fn: impl FnOnce(&mut ConfigData)) -> Result<(), GeneralError> {
        updater_fn(&mut self.config_data);
        self.save()?;
        Ok(())
    }
}
