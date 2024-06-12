use chrono::NaiveDate;

use crate::commands::{date_or_max, date_or_min};
use crate::storage::entries::Entries;

pub(crate) struct Report;

impl Report {
    pub fn invoke(
        entries: &Entries,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
        project: Option<String>,
    ) -> anyhow::Result<()> {
        let from = date_or_min(from);
        let to = date_or_max(to);
        let time_framed_entries = entries.get_in_range(from, to)?;
        let entries = match project {
            Some(project) => time_framed_entries.filter_by_project(project),
            None => time_framed_entries,
        };
        entries.report();
        Ok(())
    }
}
