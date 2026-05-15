use anyhow::{Context, Result};
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use owo_colors::OwoColorize;

use crate::config::{BehaviorConfig, Config, LogConfig, StorageConfig, StorageProvider, WeekStart};

#[allow(clippy::vec_init_then_push)] // cfg-gated pushes require this pattern
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

    // [storage] — only show backends that were compiled in.
    // The cfg-gated push calls require Vec::new() + push pattern.
    let mut storage_options: Vec<(&str, StorageProvider)> = Vec::new();
    #[cfg(feature = "storage-sqlite")]
    storage_options.push(("SQLite  (watson.db)", StorageProvider::Sqlite));
    #[cfg(feature = "storage-json")]
    storage_options.push(("JSON  (frames.json + state.json)", StorageProvider::Json));

    let labels: Vec<&str> = storage_options.iter().map(|(l, _)| *l).collect();
    let provider_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Storage provider")
        .items(&labels)
        .default(0)
        .interact()?;
    let provider = storage_options[provider_idx].1;

    println!();

    // [behavior]
    let allow_future_times = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Allow future times for start, stop and add?")
        .default(false)
        .interact()?;

    let week_start_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("First day of week (for --from week)")
        .items(["Monday", "Sunday"])
        .default(0)
        .interact()?;
    let week_start = match week_start_idx {
        1 => WeekStart::Sunday,
        _ => WeekStart::Monday,
    };

    println!();

    // [log]
    let default_limit_str: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Default number of frames shown by 'log'  (0 = show all)")
        .with_initial_text("0")
        .interact_text()?;
    let default_limit = default_limit_str.trim().parse::<usize>().unwrap_or(0);

    let config = Config {
        storage: StorageConfig {
            provider,
            data_dir: None,
        },
        behavior: BehaviorConfig {
            allow_future_times,
            week_start,
        },
        log: LogConfig { default_limit },
        epics: vec![],
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
