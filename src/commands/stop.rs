use chrono::{Local, NaiveTime, TimeZone, Utc};

use crate::storage::entries::Entries;

pub fn invoke(entries: &mut Entries, at: Option<NaiveTime>) -> anyhow::Result<()> {
    let now = Local::now();
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
    .unwrap();

    entries.set_last_end(end);
    Ok(())
}
