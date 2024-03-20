use chrono::NaiveDate;

use crate::commands::{date_or_max, date_or_min};
use crate::storage::entries::Entries;

pub fn invoke(
    entries: &Entries,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
) -> anyhow::Result<()> {
    let from = date_or_min(from);
    let to = date_or_max(to);
    let entries = entries.get_in_range(from, to)?;
    println!("{}", entries);
    Ok(())
}
