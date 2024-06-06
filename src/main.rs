use std::fs::File;

use anyhow::Context;
use clap::Parser;

use commands::Command;

use crate::commands::run;
use crate::storage::entries::Entries;
use crate::storage::get_or_create_file;

mod commands;
mod exporter;
mod storage;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config = Config::default();
    let file =
        get_or_create_file(config.file_name.clone()).context("Failed to open watson.json")?;
    let mut entries = serde_json::from_reader(&file).unwrap_or_else(|_| Entries::default());

    run(args, &mut entries).context("Failed to run command")?;

    let file = File::create(config.file_name.clone())
        .context("Failed to write result to watson.json")
        .unwrap();
    serde_json::to_writer(&file, &entries).context("Failed to write result to watson.json")?;

    Ok(())
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

struct Config {
    file_name: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            file_name: "watson.json".to_string(),
        }
    }
}
