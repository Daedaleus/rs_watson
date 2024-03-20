use chrono::{Local, NaiveTime, TimeZone, Utc};

use crate::storage::entries::Entries;
use crate::storage::entry::Entry;

pub fn invoke(
    entries: &mut Entries,
    project: String,
    tags: Option<Vec<String>>,
    at: Option<NaiveTime>,
) -> anyhow::Result<()> {
    let now = Local::now();
    let tags = tags.map(|tags| {
        tags.iter()
            .map(|tag| tag.trim_start_matches('+').to_string())
            .collect()
    });

    match at {
        Some(at) => {
            let start = now.date_naive().and_time(at);
            let start = Local.from_local_datetime(&start).unwrap();
            let start = start.with_timezone(&Utc);

            let entry = Entry::new(project, tags, start, None)?;
            println!(
                "Start logging of project {} with tags {} at {}",
                entry.get_project().clone(),
                entry.get_tags().clone().unwrap_or_default().join(", "),
                entry.get_start().clone()
            );
            entries.push(entry);
        }
        None => {
            let start = Local.from_local_datetime(&now.naive_local()).unwrap();
            let start = start.with_timezone(&Utc);

            let entry = Entry::new(project, tags, start, None)?;

            println!(
                "Start logging of project {} with tags {} at {}",
                entry.get_project().clone(),
                entry.get_tags().clone().unwrap_or_default().join(", "),
                entry.get_start().clone()
            );
            entries.push(entry);
        }
    };
    Ok(())
}
