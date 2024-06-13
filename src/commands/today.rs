use clap_derive::Args;

use crate::commands::report::Report;
use crate::commands::Invokable;
use crate::storage::entries::Entries;

#[derive(Args)]
pub struct Today;

impl Invokable for Today {
    fn invoke(&self, entries: &mut Entries) -> anyhow::Result<()> {
        let today = chrono::Local::now().naive_local().date();
        let report = Report::new(Some(today), Some(today), None);
        report.invoke(entries)
    }
}
