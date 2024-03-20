use std::fs::File;

use anyhow::Context;
use clap::Parser;

use commands::Command;

use crate::commands::run;
use crate::storage::entries::Entries;
use crate::storage::get_or_create_file;

mod commands;
mod storage;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let file_name = "watson.json";
    let file = get_or_create_file(file_name)?;
    let mut entries = serde_json::from_reader(&file).unwrap_or_else(|_| Entries::default());

    run(args, &mut entries)?;

    let file = File::create(file_name)
        .context("Failed to write result to watson.json")
        .unwrap();
    serde_json::to_writer(&file, &entries)?;

    Ok(())
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}
