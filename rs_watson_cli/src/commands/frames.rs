use anyhow::Result;
use chrono::Local;
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use owo_colors::OwoColorize;
use rs_watson::Watson;
use rs_watson_storage::Storage;

use crate::config::Config;
use crate::epic::print_epic_report;
use crate::format::{
    fmt_duration, fmt_tags, fmt_time, print_frame_summary, print_frames_grouped,
    print_report_grouped,
};
use crate::time_utils::{check_future, parse_at, prompt_time};

use super::{apply_date_filter, w_err};

pub(super) fn cmd_log<S: Storage<Error: std::error::Error + Send + Sync + 'static>>(
    watson: &Watson<S>,
    from: Option<String>,
    to: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
    config: &Config,
) -> Result<()> {
    let mut frames = apply_date_filter(
        watson.log().map_err(w_err)?,
        from,
        to,
        config.behavior.week_start,
    )?;
    let effective_limit =
        limit.or_else(|| (config.log.default_limit > 0).then_some(config.log.default_limit));
    let total = frames.len();
    let end = total.saturating_sub(offset.unwrap_or(0));
    let start = end.saturating_sub(effective_limit.unwrap_or(total));
    frames = frames[start..end].to_vec();
    if frames.is_empty() {
        println!("{}", "No frames recorded.".bright_black());
    } else {
        print_frames_grouped(&frames);
    }
    Ok(())
}

pub(super) fn cmd_today<S: Storage<Error: std::error::Error + Send + Sync + 'static>>(
    watson: &Watson<S>,
    epic: bool,
    config: &Config,
) -> Result<()> {
    let today = Local::now().date_naive();
    let frames: Vec<_> = watson
        .log()
        .map_err(w_err)?
        .into_iter()
        .filter(|f| f.start.with_timezone(&Local).date_naive() == today)
        .collect();
    if frames.is_empty() {
        println!("{}", "No frames recorded today.".bright_black());
    } else if epic {
        if config.epics.is_empty() {
            anyhow::bail!("No epics configured. Add [[epics]] entries to config.toml.");
        }
        print_epic_report(&frames, &config.epics, false);
    } else {
        print_report_grouped(&frames, false);
    }
    Ok(())
}

pub(super) fn cmd_report<S: Storage<Error: std::error::Error + Send + Sync + 'static>>(
    watson: &Watson<S>,
    from: Option<String>,
    to: Option<String>,
    epic: bool,
    config: &Config,
) -> Result<()> {
    let frames = apply_date_filter(
        watson.log().map_err(w_err)?,
        from,
        to,
        config.behavior.week_start,
    )?;
    if frames.is_empty() {
        println!("{}", "No frames recorded.".bright_black());
    } else if epic {
        if config.epics.is_empty() {
            anyhow::bail!("No epics configured. Add [[epics]] entries to config.toml.");
        }
        print_epic_report(&frames, &config.epics, true);
    } else {
        print_report_grouped(&frames, true);
    }
    Ok(())
}

pub(super) fn cmd_add<S: Storage<Error: std::error::Error + Send + Sync + 'static>>(
    watson: &Watson<S>,
    project: String,
    tags: Vec<String>,
    from: String,
    to: String,
    config: &Config,
) -> Result<()> {
    let start = parse_at(&from)?;
    let end = parse_at(&to)?;
    check_future(start, config)?;
    check_future(end, config)?;
    let frame = watson.add(&project, tags, start, end).map_err(w_err)?;
    print_frame_summary("Added   ".green().bold(), &frame);
    Ok(())
}

pub(super) fn cmd_edit<S: Storage<Error: std::error::Error + Send + Sync + 'static>>(
    watson: &Watson<S>,
) -> Result<()> {
    let frames = watson.log().map_err(w_err)?;
    if frames.is_empty() {
        println!("{}", "No frames to edit.".bright_black());
        return Ok(());
    }

    let items = frame_selector_items(&frames);
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select frame to edit")
        .items(&items)
        .default(items.len() - 1)
        .interact()?;

    let frame = &frames[selection];
    println!();

    let new_project: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Project")
        .with_initial_text(&frame.project)
        .interact_text()?;

    let tags_default = frame.tags.join(", ");
    let tags_input: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Tags  (comma-separated, empty for none)")
        .with_initial_text(&tags_default)
        .interact_text()?;
    let new_tags: Vec<String> = tags_input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let new_start = prompt_time("Start  (HH:MM or YYYY-MM-DD HH:MM)", frame.start)?;
    let new_end = prompt_time("End    (HH:MM or YYYY-MM-DD HH:MM)", frame.end)?;

    if new_end <= new_start {
        anyhow::bail!("End time must be after start time");
    }

    let updated = watson
        .edit(frame.id, new_project, new_tags, new_start, new_end)
        .map_err(w_err)?;

    println!();
    print_frame_summary("Updated ".green().bold(), &updated);
    Ok(())
}

pub(super) fn cmd_remove<S: Storage<Error: std::error::Error + Send + Sync + 'static>>(
    watson: &Watson<S>,
) -> Result<()> {
    let frames = watson.log().map_err(w_err)?;
    if frames.is_empty() {
        println!("{}", "No frames to remove.".bright_black());
        return Ok(());
    }

    let items = frame_selector_items(&frames);
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select frame to remove")
        .items(&items)
        .default(items.len() - 1)
        .interact()?;

    let frame = &frames[selection];
    let confirmed = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Remove \"{}\" ({} → {})?",
            frame.project,
            fmt_time(frame.start),
            fmt_time(frame.end),
        ))
        .default(false)
        .interact()?;

    if !confirmed {
        println!("{}", "Aborted.".bright_black());
        return Ok(());
    }

    watson.remove(frame.id).map_err(w_err)?;
    println!(
        "{} {}{}",
        "Removed ".red().bold(),
        frame.project.yellow().bold(),
        fmt_tags(&frame.tags),
    );
    Ok(())
}

/// Builds the display strings for the interactive frame selector used by edit and remove.
fn frame_selector_items(frames: &[rs_watson::Frame]) -> Vec<String> {
    frames
        .iter()
        .map(|f| {
            format!(
                "{}  {} → {}  {:<10}  {}{}",
                f.start.format("%Y-%m-%d"),
                fmt_time(f.start),
                fmt_time(f.end),
                fmt_duration(f.end - f.start),
                f.project,
                if f.tags.is_empty() {
                    String::new()
                } else {
                    format!("  [{}]", f.tags.join(", "))
                },
            )
        })
        .collect()
}
