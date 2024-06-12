use anyhow::{Context, Error};
use chrono::{DateTime, Local, NaiveTime, TimeZone, Utc};
use colored::Colorize;

use crate::storage::entries::Entries;

pub(crate) struct Stop;

impl Stop {
    pub fn invoke(entries: &mut Entries, at: Option<NaiveTime>) -> anyhow::Result<()> {
        let now = Local::now();

        Self::handle_command(entries, at, now)?;
        Ok(())
    }

    pub fn handle_command(
        entries: &mut Entries,
        at: Option<NaiveTime>,
        now: DateTime<Local>,
    ) -> Result<(), Error> {
        println!(
            "Stop logging at {}",
            now.format("%H:%M:%S").to_string().cyan()
        );
        let end = match at {
            Some(at) => {
                let end = now.date_naive().and_time(at);
                let end = Local.from_local_datetime(&end).unwrap();
                let end = end.with_timezone(&Utc);
                Some(end)
            }
            None => {
                let end = now.with_timezone(&Utc);
                Some(end)
            }
        }
        .context("Failed to parse time")?;

        entries.set_last_end(end);
        Ok(())
    }
}
