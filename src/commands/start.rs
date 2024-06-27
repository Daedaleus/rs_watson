use anyhow::Error;
use chrono::{DateTime, Local, TimeZone, Utc};
use clap_derive::Args;
use colored::Colorize;

use crate::commands::params::{At, Project, Tags};
use crate::commands::parse_time;
use crate::commands::Invokable;
use crate::storage::entries::Entries;
use crate::storage::entry::Entry;

#[derive(Args)]
pub struct Start {
    project: Project,
    tags: Option<Tags>,
    #[arg(short = 'a', value_parser = parse_time)]
    at: Option<At>,
}

impl Invokable for Start {
    fn invoke(&self, entries: &mut Entries) -> anyhow::Result<()> {
        let now = Local::now();
        let tags = self.tags.clone();
        match entries.get_last() {
            Some(entry) => {
                if entry.is_running() {
                    crate::commands::stop::Stop::handle_command(entries, self.at.clone(), now)?;
                    Self::handle_command(
                        entries,
                        self.project.clone(),
                        self.at.clone(),
                        &now,
                        &tags,
                    )?;
                } else {
                    Self::handle_command(
                        entries,
                        self.project.clone(),
                        self.at.clone(),
                        &now,
                        &tags,
                    )?
                }
            }
            None => {
                Self::handle_command(entries, self.project.clone(), self.at.clone(), &now, &tags)?
            }
        }
        Ok(())
    }
}

impl Start {
    fn handle_command(
        entries: &mut Entries,
        project: Project,
        at: Option<At>,
        now: &DateTime<Local>,
        tags: &Option<Tags>,
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
        project: Project,
        now: &DateTime<Local>,
        tags: Option<Tags>,
    ) -> Result<(), Error> {
        let start = Local.from_local_datetime(&now.naive_local()).unwrap();
        let start = start.with_timezone(&Utc);

        let entry = Entry::new(project, tags, start, None)?;

        println!(
            "Start logging of project {} with tags {} at {}",
            entry.get_project().clone().bright_green(),
            entry
                .get_tags()
                .clone()
                .unwrap_or_default()
                .as_string()
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
        project: Project,
        now: &DateTime<Local>,
        tags: &Option<Tags>,
        at: At,
    ) -> Result<(), Error> {
        let start = now.date_naive().and_time(at.into());
        let start = Local.from_local_datetime(&start).unwrap();
        let start = start.with_timezone(&Utc);

        let entry = Entry::new(project, tags.clone(), start, None)?;
        println!(
            "Start logging of project {} with tags {} at {}",
            entry.get_project().clone().cyan(),
            entry
                .get_tags()
                .clone()
                .unwrap_or_default()
                .as_string()
                .green(),
            entry.get_start().clone()
        );
        entries.push(entry);
        Ok(())
    }
}
