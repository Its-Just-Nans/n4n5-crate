use std::{
    fs::{create_dir_all, read_to_string, File},
    io::Write,
    path::PathBuf,
    str,
};

use home::home_dir;
use serde::{Deserialize, Serialize};

use crate::commands::movies::Movies;

#[derive(Deserialize, Default)]
pub struct Config {
    pub debug: Option<u8>,
    pub config_path: PathBuf,
    pub config_data: ConfigData,
}

#[derive(Deserialize, Serialize, Default)]
pub struct ConfigData {
    pub movies: Option<Movies>,
}

impl Config {
    fn parse_config(str_config: &str, path_config: PathBuf) -> Config {
        Config {
            debug: None,
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

    pub fn new() -> Config {
        let config_path = Config::get_config_path();
        let contents = read_to_string(config_path.clone())
            .expect(&format!("Unable to open {:?}", config_path));
        Config::parse_config(&contents, config_path)
    }

    pub fn save(&self) {
        let config_str = toml::to_string(&self.config_data).expect("Unable to serialize config");
        let mut file = File::create(&self.config_path).expect("Unable to create config file");
        file.write_all(config_str.as_bytes())
            .expect("Unable to write to config file");
    }

    pub fn new_from_path(custom_path: &PathBuf) -> Config {
        let contents = read_to_string(custom_path.clone())
            .expect(&format!("Unable to open {:?}", custom_path));
        Config::parse_config(&contents, custom_path.clone())
    }

    pub fn set_debug(&mut self, value: &u8) {
        self.debug = Some(value.clone());
    }

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
            file.write(b"").expect("Unable to write to config file");
        }
        return config_path;
    }

    pub fn update(&mut self, updater_fn: impl FnOnce(&mut ConfigData) -> &mut ConfigData) {
        updater_fn(&mut self.config_data);
        self.save();
    }
}
