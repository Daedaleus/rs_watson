use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub storage: StorageConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StorageConfig {
    pub provider: StorageProvider,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum StorageProvider {
    Json,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            storage: StorageConfig {
                provider: StorageProvider::Json,
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = dirs::config_dir()
            .context("Could not determine config directory")?
            .join("rs_watson")
            .join("config.toml");

        if !path.exists() {
            return Ok(Config::default());
        }

        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Could not read config file: {}", path.display()))?;

        toml::from_str(&content)
            .with_context(|| format!("Invalid config file: {}", path.display()))
    }
}
