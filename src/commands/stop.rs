use anyhow::{Context, Error};
use chrono::{DateTime, Local, NaiveTime, TimeZone, Utc};
use clap_derive::Args;
use colored::Colorize;

use crate::commands::parse_time;
use crate::commands::Invokable;
use crate::storage::entries::Entries;

#[derive(Args)]
pub struct Stop {
    #[clap(short = 'a')]
    #[arg(value_parser(parse_time))]
    at: Option<NaiveTime>,
}

impl Invokable for Stop {
    fn invoke(&self, entries: &mut Entries) -> anyhow::Result<()> {
        let now = Local::now();
        Self::handle_command(entries, self.at, now)?;
        Ok(())
    }
}

impl Stop {
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
