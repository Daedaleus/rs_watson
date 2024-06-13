use anyhow::Error;
use chrono::{DateTime, Local, NaiveTime, TimeZone, Utc};
use clap_derive::Args;
use colored::Colorize;

use crate::commands::parse_time;
use crate::commands::Invokable;
use crate::storage::entries::Entries;
use crate::storage::entry::Entry;

#[derive(Args)]
pub struct Start {
    project: String,
    tags: Option<Vec<String>>,
    #[clap(short = 'a', value_parser = parse_time)]
    at: Option<NaiveTime>,
}

impl Invokable for Start {
    fn invoke(&self, entries: &mut Entries) -> anyhow::Result<()> {
        let now = Local::now();
        let tags = Self::extract_tags(self.tags.clone());
        match entries.get_last() {
            Some(entry) => {
                if entry.is_running() {
                    crate::commands::stop::Stop::handle_command(entries, self.at, now)?;
                    Self::handle_command(entries, &self.project, self.at, &now, &tags)?;
                } else {
                    Self::handle_command(entries, &self.project, self.at, &now, &tags)?
                }
            }
            None => Self::handle_command(entries, &self.project, self.at, &now, &tags)?,
        }
        Ok(())
    }
}

impl Start {
    fn handle_command(
        entries: &mut Entries,
        project: &str,
        at: Option<NaiveTime>,
        now: &DateTime<Local>,
        tags: &Option<Vec<String>>,
    ) -> Result<(), Error> {
        match at {
            Some(at) => {
                Self::with_start(entries, project, now, tags, at)?;
            }
            None => {
                Self::without_start(entries, project, now, tags.clone())?;
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
            entry.get_project().clone().bright_green(),
            entry
                .get_tags()
                .clone()
                .unwrap_or_default()
                .join(", ")
                .bright_green(),
            entry
                .get_start()
                .clone()
                .format("%H:%M:%S")
                .to_string()
                .cyan()
                .to_string()
                .green()
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
}

#[cfg(test)]
mod tests {
    use crate::commands::start::Start;

    #[test]
    fn test_extract_tags() {
        let tags = Some(vec!["+tag1".to_string(), "+tag2".to_string()]);
        let extracted_tags = Start::extract_tags(tags);
        assert_eq!(
            extracted_tags,
            Some(vec!["tag1".to_string(), "tag2".to_string()])
        );
    }
}
