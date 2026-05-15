use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;
use rs_watson::Watson;
use rs_watson_storage::Storage;

use crate::config::Config;
use crate::format::{fmt_duration, fmt_tags, fmt_time};
use crate::time_utils::{check_future, parse_at};
use rs_watson::StartResult;

use super::w_err;

pub(super) fn cmd_start<S: Storage<Error: std::error::Error + Send + Sync + 'static>>(
    watson: &Watson<S>,
    project: String,
    tags: Vec<String>,
    at: Option<String>,
    config: &Config,
) -> Result<()> {
    let time = at
        .map(|s| parse_at(&s))
        .transpose()?
        .unwrap_or_else(Utc::now);
    check_future(time, config)?;
    let StartResult { replaced, active } = watson
        .start_or_replace(&project, tags, time)
        .map_err(w_err)?;

    if let Some(stopped) = replaced {
        println!(
            "{} {}{}",
            "Stopped ".red().bold(),
            stopped.project.yellow().bold(),
            fmt_tags(&stopped.tags),
        );
        println!(
            "  {}  {}  {}",
            fmt_time(stopped.start).bright_white(),
            "→".white(),
            fmt_time(stopped.end).bright_white(),
        );
        println!(
            "  {}",
            fmt_duration(stopped.end - stopped.start).magenta().bold()
        );
        println!();
    }

    println!(
        "{} {}{}  {}",
        "Starting".green().bold(),
        active.project.yellow().bold(),
        fmt_tags(&active.tags),
        fmt_time(active.start).bright_black(),
    );
    Ok(())
}

pub(super) fn cmd_stop<S: Storage<Error: std::error::Error + Send + Sync + 'static>>(
    watson: &Watson<S>,
    at: Option<String>,
    config: &Config,
) -> Result<()> {
    let time = at
        .map(|s| parse_at(&s))
        .transpose()?
        .unwrap_or_else(Utc::now);
    check_future(time, config)?;
    let frame = watson.stop(time).map_err(w_err)?;
    println!(
        "{} {}{}",
        "Stopped ".red().bold(),
        frame.project.yellow().bold(),
        fmt_tags(&frame.tags),
    );
    println!(
        "  {}  {}  {}",
        fmt_time(frame.start).bright_white(),
        "→".white(),
        fmt_time(frame.end).bright_white(),
    );
    println!(
        "  {}",
        fmt_duration(frame.end - frame.start).magenta().bold()
    );
    Ok(())
}

pub(super) fn cmd_cancel<S: Storage<Error: std::error::Error + Send + Sync + 'static>>(
    watson: &Watson<S>,
) -> Result<()> {
    let frame = watson.cancel().map_err(w_err)?;
    let discarded = fmt_duration(Utc::now() - frame.start);
    println!(
        "{} {}{}",
        "Cancelled".red().bold(),
        frame.project.yellow().bold(),
        fmt_tags(&frame.tags),
    );
    println!(
        "  {} {}  {} {}",
        "started".bright_black(),
        fmt_time(frame.start).bright_white(),
        discarded.magenta().bold(),
        "discarded".bright_black(),
    );
    Ok(())
}

pub(super) fn cmd_status<S: Storage<Error: std::error::Error + Send + Sync + 'static>>(
    watson: &Watson<S>,
) -> Result<()> {
    let now = Utc::now();
    match watson.status().map_err(w_err)? {
        Some(frame) => {
            println!(
                "{} {}{}",
                "Tracking".blue().bold(),
                frame.project.yellow().bold(),
                fmt_tags(&frame.tags),
            );
            println!(
                "  {} {}  {}",
                "since".bright_black(),
                fmt_time(frame.start).bright_white(),
                fmt_duration(now - frame.start).magenta().bold(),
            );
        }
        None => println!("{}", "Not tracking anything.".bright_black()),
    }
    Ok(())
}
