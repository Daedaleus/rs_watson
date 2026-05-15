use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

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

// --- [storage] -------------------------------------------------------------

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct StorageConfig {
    /// Storage backend. Default: json.
    #[serde(default)]
    pub provider: StorageProvider,
    /// Custom data directory. Overridden by RS_WATSON_DATA_DIR env var.
    #[serde(default)]
    pub data_dir: Option<String>,
}

#[cfg(not(any(feature = "storage-json", feature = "storage-sqlite")))]
compile_error!(
    "rs_watson_cli: at least one storage backend must be enabled \
     (features: storage-json, storage-sqlite)"
);

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
        // SQLite takes priority when both are compiled in.
        #[cfg(feature = "storage-sqlite")]
        return StorageProvider::Sqlite;
        #[cfg(all(feature = "storage-json", not(feature = "storage-sqlite")))]
        return StorageProvider::Json;
    }
}

// --- [behavior] ------------------------------------------------------------

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

// --- [log] -----------------------------------------------------------------

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct LogConfig {
    /// Default number of frames shown by `watson log` (0 = show all). Default: 0.
    #[serde(default)]
    pub default_limit: usize,
}

// --- [epics] ---------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize)]
pub struct EpicConfig {
    pub name: String,
    pub project: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

// --- Loading ---------------------------------------------------------------

impl Config {
    pub fn load() -> Result<Self> {
        let path = if let Ok(dir) = std::env::var("RS_WATSON_CONFIG_DIR") {
            std::path::PathBuf::from(dir).join("config.toml")
        } else {
            dirs::config_dir()
                .context("Could not determine config directory")?
                .join("rs_watson")
                .join("config.toml")
        };

        if !path.exists() {
            return Ok(Config::default());
        }

        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Could not read config file: {}", path.display()))?;

        toml::from_str(&content).with_context(|| format!("Invalid config file: {}", path.display()))
    }
}
