use crate::commands::{date_or_max, date_or_min, Command, Invokable};
use crate::storage::entries::Entries;

pub(crate) struct Log;

impl Invokable for Log {
    fn invoke(&self, entries: &mut Entries, params: Command) -> anyhow::Result<()> {
        if let Command::Log { from, to } = params {
            let from = date_or_min(from);
            let to = date_or_max(to);
            let entries = entries.get_in_range(from, to)?;
            println!("{}", entries);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Invalid parameters"))
        }
    }
}
