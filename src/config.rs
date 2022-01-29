use directories_next::ProjectDirs;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub json_path: String,
}

pub fn load() -> anyhow::Result<Config> {
    let workspace = PathBuf::from("settings.toml");
    let xdg_home = ProjectDirs::from("de", "pkeil", "rs_watson")
        .expect("Home directory could not be detected")
        .config_dir()
        .join("settings.toml");

    let file = if workspace.exists() {
        std::fs::read_to_string(workspace)?
    } else {
        std::fs::read_to_string(xdg_home)?
    };

    let config = toml::from_str(&file)?;

    Ok(config)
}
