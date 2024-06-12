use std::fs::File;

use anyhow::{Context, Result};
use clap::Parser;

use commands::Command;

use crate::commands::run;
use crate::config::Config;
use crate::storage::entries::Entries;
use crate::storage::get_or_create_file;

mod commands;
mod config;
mod exporter;
mod storage;

fn main() -> Result<()> {
    let args = Args::parse();
    let config = Config::load_or_default()?;
    let file = get_or_create_file(config.get_file_name()).context("Failed to open watson.json")?;
    let mut entries = serde_json::from_reader(&file).unwrap_or_else(|_| Entries::default());

    run(args, &mut entries).context("Failed to run command")?;

    let file = File::create(config.get_file_name())
        .context("Failed to write result to watson.json")
        .unwrap();
    serde_json::to_writer_pretty(&file, &entries)
        .context("Failed to write result to watson.json")?;

    Ok(())
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}
