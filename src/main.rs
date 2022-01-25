use log::LevelFilter;

use crate::cli::get_args;
use crate::models::frame::Frame;
use crate::storing_engines::json::{read, write};

mod storing_engines;
mod models;
mod cli;

fn main() -> anyhow::Result<()> {
    setup_logging();
    let cli = get_args();
    log::debug!("{:#?}", cli);


    let frame = Frame { project: cli.project, task: cli.task, from: cli.from };
    write(frame, "frames.json".to_string())?;

    let frames = read("frames.json".to_string())?;
    log::debug!("{:#?}", frames);
    Ok(())
}

fn setup_logging() {
    env_logger::Builder::from_default_env()
        .filter(None, LevelFilter::Debug)
        .init();
}

