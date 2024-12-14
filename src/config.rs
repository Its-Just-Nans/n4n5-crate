use std::{
    fs::{create_dir_all, read_to_string, File},
    io::Write,
    path::PathBuf,
    str,
};

use home::home_dir;
use serde::{Deserialize, Serialize};

use crate::commands::{gh::lib::Gh, movies::Movies, sync::SyncCliCommand};

/// Configuration object
/// It's linked to a configuration file
#[derive(Deserialize, Default)]
pub struct Config {
    /// debug level
    pub debug: u8,

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

    /// Settings configuration
    pub sync: Option<SyncCliCommand>,

    /// Github configuration
    pub gh: Option<Gh>,
}

impl Config {
    /// Parse the config file
    fn parse_config(str_config: &str, path_config: PathBuf) -> Config {
        Config {
            debug: 0,
            config_path: path_config,
            config_data: match toml::from_str(str_config) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("Unable to parse config file: {:?}", e);
                    eprintln!("Using default config");
                    ConfigData::default()
                }
            },
        }
    }

    /// Create a new Config object from the default path
    pub fn new() -> Config {
        let config_path = Config::get_config_path();
        let contents = read_to_string(config_path.clone())
            .unwrap_or_else(|_| panic!("Unable to open {:?}", config_path));
        Config::parse_config(&contents, config_path)
    }

    /// Save the config data to the config file
    pub fn save(&self) {
        let config_str = toml::to_string(&self.config_data).expect("Unable to serialize config");
        let mut file = File::create(&self.config_path).expect("Unable to create config file");
        file.write_all(config_str.as_bytes())
            .expect("Unable to write to config file");
    }

    /// Create a new Config object from a custom path
    pub fn new_from_path(custom_path: &PathBuf) -> Config {
        let contents = read_to_string(custom_path.clone())
            .unwrap_or_else(|_| panic!("Unable to open {:?}", custom_path));
        Config::parse_config(&contents, custom_path.clone())
    }

    /// Set the debug value
    pub fn set_debug(&mut self, value: u8) {
        self.debug = value;
    }

    /// Get the path to the config file
    pub fn get_config_path() -> PathBuf {
        let home_dir = match home_dir() {
            Some(path) if !path.as_os_str().is_empty() => Ok(path),
            _ => Err(()),
        }
        .expect("Unable to get your home dir! home::home_dir() isn't working");
        let config_directory = home_dir.join(".config").join(".n4n5");
        let config_path = config_directory.join("config.toml");
        create_dir_all(config_directory).expect("Unable to create config dir");
        if !config_path.exists() {
            let mut file = File::create(&config_path).expect("Unable to create config file");
            file.write_all(b"").expect("Unable to write to config file");
        }
        config_path
    }

    /// Update the config data and save it to the config file
    pub fn update(&mut self, updater_fn: impl FnOnce(&mut ConfigData) -> &mut ConfigData) {
        updater_fn(&mut self.config_data);
        self.save();
    }
}
