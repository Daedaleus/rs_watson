use std::path::PathBuf;

use anyhow::Context;
use clap::Error;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

enum LocationType {
    Config,
    Data,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    files: Files,
}

#[derive(Serialize, Deserialize, Clone)]
struct Files {
    file_name: String,
    config_name: String,
}

impl Default for Config {
    fn default() -> Self {
        let file_path =
            PathBuf::from_iter([Config::get_location(LocationType::Data), "watson".into()])
                .with_extension("json");
        let file_path = file_path.to_str().unwrap();

        let config_path =
            PathBuf::from_iter([Config::get_location(LocationType::Config), "config".into()])
                .with_extension("toml");
        let config_path = config_path.to_str().unwrap();

        Self {
            files: Files {
                file_name: file_path.into(),
                config_name: config_path.into(),
            },
        }
    }
}

impl Config {
    pub(crate) fn load_or_default() -> anyhow::Result<Self, Error> {
        let config = Config::default();
        let config_string = std::fs::read_to_string(config.get_config_name());
        match config_string {
            Ok(config_string) => {
                let config: Config = toml::from_str(&config_string)
                    .context("Failed to parse config.toml")
                    .unwrap();
                Ok(config)
            }
            Err(_) => {
                config.save().context("Failed to save config.toml").unwrap();
                Ok(config)
            }
        }
    }

    fn save(&self) -> anyhow::Result<(), Error> {
        let config_string = toml::to_string(self)
            .context("Failed to serialize config.toml")
            .unwrap();
        std::fs::write(self.get_config_name(), config_string)
            .context("Failed to write config.toml")
            .unwrap();
        Ok(())
    }

    fn get_location(location_type: LocationType) -> String {
        let root_dir = ProjectDirs::from("de", "daedaleus", "rswatson")
            .context("Failed to get project directories")
            .unwrap();
        match location_type {
            LocationType::Config => {
                let path = root_dir.config_dir().to_str().unwrap().to_string();
                Self::create_location(path)
            }
            LocationType::Data => {
                let path = root_dir.data_dir().to_str().unwrap().to_string();
                Self::create_location(path)
            }
        }
    }

    fn create_location(path: String) -> String {
        if !std::path::Path::new(&path).exists() {
            std::fs::create_dir_all(&path)
                .context("Failed to create config directory")
                .unwrap();
        }
        path
    }

    pub fn get_file_name(&self) -> String {
        self.files.file_name.clone()
    }

    pub fn get_config_name(&self) -> String {
        self.files.config_name.clone()
    }
}
