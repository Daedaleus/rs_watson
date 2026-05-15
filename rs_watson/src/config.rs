use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::Frame;

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Could not determine config directory")]
    NoConfigDir,
    #[error("Could not read config file {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("Invalid config file {path}: {source}")]
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },
}

// ---------------------------------------------------------------------------
// Top-level Config
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Config {
    #[serde(default)]
    pub storage: StorageConfig,
    #[serde(default)]
    pub behavior: BehaviorConfig,
    #[serde(default)]
    pub log: LogConfig,
    #[serde(default)]
    pub epics: Vec<EpicConfig>,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let path = if let Ok(dir) = std::env::var("RS_WATSON_CONFIG_DIR") {
            PathBuf::from(dir).join("config.toml")
        } else {
            dirs::config_dir()
                .ok_or(ConfigError::NoConfigDir)?
                .join("rs_watson")
                .join("config.toml")
        };

        if !path.exists() {
            return Ok(Config::default());
        }

        let content = std::fs::read_to_string(&path).map_err(|source| ConfigError::Read {
            path: path.clone(),
            source,
        })?;

        toml::from_str(&content).map_err(|source| ConfigError::Parse { path, source })
    }
}

// ---------------------------------------------------------------------------
// [storage]
// ---------------------------------------------------------------------------

#[cfg(not(any(feature = "storage-json", feature = "storage-sqlite")))]
compile_error!(
    "rs_watson: at least one storage backend must be enabled \
     (features: storage-json, storage-sqlite)"
);

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct StorageConfig {
    #[serde(default)]
    pub provider: StorageProvider,
    /// Custom data directory. Overridden by RS_WATSON_DATA_DIR env var.
    #[serde(default)]
    pub data_dir: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum StorageProvider {
    #[cfg(feature = "storage-json")]
    Json,
    #[cfg(feature = "storage-sqlite")]
    Sqlite,
}

impl Default for StorageProvider {
    fn default() -> Self {
        #[cfg(feature = "storage-sqlite")]
        return StorageProvider::Sqlite;
        #[cfg(all(feature = "storage-json", not(feature = "storage-sqlite")))]
        return StorageProvider::Json;
    }
}

// ---------------------------------------------------------------------------
// [behavior]
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct BehaviorConfig {
    /// Allow start, stop and add to accept times in the future. Default: false.
    #[serde(default)]
    pub allow_future_times: bool,
    /// First day of the week for the "week" date shortcut. Default: monday.
    #[serde(default)]
    pub week_start: WeekStart,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WeekStart {
    #[default]
    Monday,
    Sunday,
}

// ---------------------------------------------------------------------------
// [log]
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct LogConfig {
    /// Default number of frames shown by `log` (0 = show all). Default: 0.
    #[serde(default)]
    pub default_limit: usize,
}

// ---------------------------------------------------------------------------
// [epics]
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize)]
pub struct EpicConfig {
    pub name: String,
    pub project: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

// ---------------------------------------------------------------------------
// Epic resolution
// ---------------------------------------------------------------------------

/// Returns the best-matching epic name for a frame.
/// "Best" means the epic with the most matching tags (most specific rule).
/// All epic tags must be present in the frame; epic project must match exactly.
pub fn resolve_epic<'a>(frame: &Frame, epics: &'a [EpicConfig]) -> Option<&'a str> {
    epics
        .iter()
        .filter(|e| e.project == frame.project)
        .filter(|e| e.tags.iter().all(|t| frame.tags.contains(t)))
        .max_by_key(|e| e.tags.len())
        .map(|e| e.name.as_str())
}
