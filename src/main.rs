use log::LevelFilter;

use crate::cli::parse;
use crate::models::frame::Frame;
use crate::storing_engines::json::{read, write};

mod cli;
mod models;
mod storing_engines;

pub mod config;

fn main() -> anyhow::Result<()> {
    setup_logging();
    parse()?;
    Ok(())
}

fn setup_logging() {
    env_logger::Builder::from_default_env()
        .filter(None, LevelFilter::Debug)
        .init();
}
