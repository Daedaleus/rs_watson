use crate::commands::{date_or_max, date_or_min, Command, Invokable};
use crate::storage::entries::Entries;

pub(crate) struct Report;

impl Invokable for Report {
    fn invoke(&self, entries: &mut Entries, params: Command) -> anyhow::Result<()> {
        if let Command::Report { from, to, project } = params {
            let from = date_or_min(from);
            let to = date_or_max(to);
            let time_framed_entries = entries.get_in_range(from, to)?;
            let entries = match project {
                Some(project) => time_framed_entries.filter_by_project(project),
                None => time_framed_entries,
            };
            entries.report();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Invalid parameters"))
        }
    }
}
