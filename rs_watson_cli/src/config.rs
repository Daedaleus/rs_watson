use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Config {
    #[serde(default)]
    pub storage: StorageConfig,
    #[serde(default)]
    pub behavior: BehaviorConfig,
}

// --- [storage] -------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize)]
pub struct StorageConfig {
    pub provider: StorageProvider,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum StorageProvider {
    Json,
    Sqlite,
}

impl Default for StorageConfig {
    fn default() -> Self {
        StorageConfig {
            provider: StorageProvider::Json,
        }
    }
}

// --- [behavior] ------------------------------------------------------------

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct BehaviorConfig {
    /// Allow start, stop and add to accept times in the future. Default: false.
    #[serde(default)]
    pub allow_future_times: bool,
}

// --- Loading ---------------------------------------------------------------

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

        toml::from_str(&content).with_context(|| format!("Invalid config file: {}", path.display()))
    }
}
