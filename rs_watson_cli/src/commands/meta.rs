use anyhow::Result;
use owo_colors::OwoColorize;
use rs_watson::Watson;
use rs_watson_storage::Storage;

use crate::config::Config;

use super::w_err;

pub(super) fn cmd_projects<S: Storage<Error: std::error::Error + Send + Sync + 'static>>(
    watson: &Watson<S>,
) -> Result<()> {
    let projects = watson.projects().map_err(w_err)?;
    if projects.is_empty() {
        println!("{}", "No projects recorded yet.".bright_black());
    } else {
        for name in &projects {
            println!("{}", name.yellow().bold());
        }
    }
    Ok(())
}

pub(super) fn cmd_tags<S: Storage<Error: std::error::Error + Send + Sync + 'static>>(
    watson: &Watson<S>,
) -> Result<()> {
    let tags = watson.tags().map_err(w_err)?;
    if tags.is_empty() {
        println!("{}", "No tags recorded yet.".bright_black());
    } else {
        for tag in &tags {
            println!("{}", tag.cyan());
        }
    }
    Ok(())
}

pub(super) fn cmd_epics(config: &Config) -> Result<()> {
    if config.epics.is_empty() {
        println!(
            "{}",
            "No epics configured. Add [[epics]] entries to config.toml.".bright_black()
        );
        return Ok(());
    }
    for epic in &config.epics {
        println!("{}", epic.name.cyan().bold());
        println!("  project  {}", epic.project.yellow());
        if epic.tags.is_empty() {
            println!("  tags     {}", "(any)".bright_black());
        } else {
            println!("  tags     {}", epic.tags.join(", ").cyan());
        }
    }
    Ok(())
}

pub(super) fn cmd_rename<S: Storage<Error: std::error::Error + Send + Sync + 'static>>(
    watson: &Watson<S>,
    from: String,
    to: String,
) -> Result<()> {
    let count = watson.rename(&from, &to).map_err(w_err)?;
    println!(
        "{}  {}  {}  {} {}",
        "Renamed".green().bold(),
        from.yellow().bold(),
        "→".white(),
        to.yellow().bold(),
        format!("({count} updated)").bright_black(),
    );
    Ok(())
}
