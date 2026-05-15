use anyhow::{Context, Result};
use dialoguer::{Confirm, Select, theme::ColorfulTheme};
use owo_colors::OwoColorize;

use crate::config::{BehaviorConfig, Config, StorageConfig, StorageProvider};

pub(crate) fn cmd_init() -> Result<()> {
    let config_dir = dirs::config_dir()
        .context("Could not determine config directory")?
        .join("rs_watson");
    let config_path = config_dir.join("config.toml");

    if config_path.exists() {
        let overwrite = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Config file already exists. Overwrite?")
            .default(false)
            .interact()?;
        if !overwrite {
            println!("{}", "Aborted.".bright_black());
            return Ok(());
        }
    }

    println!();

    let provider_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Storage provider")
        .items(["JSON  (frames.json + state.json)", "SQLite  (watson.db)"])
        .default(0)
        .interact()?;
    let provider = match provider_idx {
        1 => StorageProvider::Sqlite,
        _ => StorageProvider::Json,
    };

    println!();

    let allow_future_times = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Allow future times for start, stop and add?")
        .default(false)
        .interact()?;

    let config = Config {
        storage: StorageConfig {
            provider,
            data_dir: None,
        },
        behavior: BehaviorConfig { allow_future_times },
    };

    std::fs::create_dir_all(&config_dir).with_context(|| {
        format!(
            "Could not create config directory: {}",
            config_dir.display()
        )
    })?;

    let content = toml::to_string(&config).context("Could not serialize config")?;
    std::fs::write(&config_path, &content)
        .with_context(|| format!("Could not write config: {}", config_path.display()))?;

    println!();
    println!(
        "{} {}",
        "Config written to".green().bold(),
        config_path.display().to_string().bright_white(),
    );
    println!();
    for line in content.lines() {
        println!("  {}", line.bright_black());
    }

    Ok(())
}
