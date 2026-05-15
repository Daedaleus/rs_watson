mod commands;
mod config;
mod format;
mod time_utils;

use std::process;

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser};
use owo_colors::OwoColorize;
use rs_watson::Watson;
use rs_watson_storage::json::JsonStorage;
use rs_watson_storage::sqlite::SqliteStorage;

use crate::commands::{Commands, cmd_init, dispatch};
use crate::config::{Config, StorageProvider};

#[derive(Parser)]
#[command(name = "watson", about = "Time tracking tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {}", "Error:".red().bold(), e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    if matches!(cli.command, Commands::Init) {
        return cmd_init();
    }

    if let Commands::Completions { shell } = cli.command {
        clap_complete::generate(shell, &mut Cli::command(), "watson", &mut std::io::stdout());
        return Ok(());
    }

    let config = Config::load()?;

    let data_dir = if let Ok(dir) = std::env::var("RS_WATSON_DATA_DIR") {
        std::path::PathBuf::from(dir)
    } else if let Some(dir) = &config.storage.data_dir {
        std::path::PathBuf::from(dir)
    } else {
        dirs::data_dir()
            .context("Could not determine data directory")?
            .join("rs_watson")
    };
    std::fs::create_dir_all(&data_dir)
        .with_context(|| format!("Could not create data directory: {}", data_dir.display()))?;

    match config.storage.provider {
        StorageProvider::Json => {
            dispatch(
                Watson::new(JsonStorage::new(&data_dir)),
                cli.command,
                &config,
            )?;
        }
        StorageProvider::Sqlite => {
            let storage = SqliteStorage::new(data_dir.join("watson.db"))
                .context("Could not open SQLite database")?;
            dispatch(Watson::new(storage), cli.command, &config)?;
        }
    }

    Ok(())
}
