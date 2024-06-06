use anyhow::Error;
use chrono::{DateTime, Local, NaiveTime, TimeZone, Utc};
use colored::Colorize;

use crate::storage::entries::Entries;
use crate::storage::entry::Entry;

pub fn invoke(
    entries: &mut Entries,
    project: String,
    tags: Option<Vec<String>>,
    at: Option<NaiveTime>,
) -> anyhow::Result<()> {
    let now = Local::now();
    let tags = extract_tags(tags);
    match entries.get_last().unwrap().is_running() {
        true => {
            crate::commands::stop::handle_command(entries, at, now)?;
            handle_command(entries, &project, at, &now, &tags)?;
        }
        false => handle_command(entries, &project, at, &now, &tags)?,
    }

    Ok(())
}

fn handle_command(
    entries: &mut Entries,
    project: &str,
    at: Option<NaiveTime>,
    now: &DateTime<Local>,
    tags: &Option<Vec<String>>,
) -> Result<(), Error> {
    match at {
        Some(at) => {
            with_start(entries, project, now, tags, at)?;
        }
        None => {
            without_start(entries, project, now, tags.clone())?;
        }
    };
    Ok(())
}

fn without_start(
    entries: &mut Entries,
    project: impl Into<String>,
    now: &DateTime<Local>,
    tags: Option<Vec<String>>,
) -> Result<(), Error> {
    let start = Local.from_local_datetime(&now.naive_local()).unwrap();
    let start = start.with_timezone(&Utc);

    let entry = Entry::new(project.into(), tags, start, None)?;

    println!(
        "Start logging of project {} with tags {} at {}",
        entry.get_project().clone(),
        entry.get_tags().clone().unwrap_or_default().join(", "),
        entry.get_start().clone()
    );
    entries.push(entry);
    Ok(())
}

fn with_start(
    entries: &mut Entries,
    project: &str,
    now: &DateTime<Local>,
    tags: &Option<Vec<String>>,
    at: NaiveTime,
) -> Result<(), Error> {
    let start = now.date_naive().and_time(at);
    let start = Local.from_local_datetime(&start).unwrap();
    let start = start.with_timezone(&Utc);

    let entry = Entry::new(project.into(), tags.clone(), start, None)?;
    println!(
        "Start logging of project {} with tags {} at {}",
        entry.get_project().clone().cyan(),
        entry
            .get_tags()
            .clone()
            .unwrap_or_default()
            .join(", ")
            .green(),
        entry.get_start().clone()
    );
    entries.push(entry);
    Ok(())
}

fn extract_tags(tags: Option<Vec<String>>) -> Option<Vec<String>> {
    let tags = tags.map(|tags| {
        tags.iter()
            .map(|tag| tag.trim_start_matches('+').to_string())
            .collect()
    });
    tags
}
