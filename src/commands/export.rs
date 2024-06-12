use crate::commands::{Command, date_or_max, date_or_min, Invokable};
use crate::exporter::csv::CsvExporter;
use crate::exporter::Exporter;
use crate::storage::entries::Entries;

pub(crate) struct Export;

impl Invokable for Export {
    fn invoke(&self, entries: &mut Entries, params: Command) -> anyhow::Result<()> {
        if let Command::Export {
            from,
            to,
            path,
            export_args,
        } = params
        {
            let from = date_or_min(from);
            let to = date_or_max(to);
            let entries = entries.get_in_range(from, to)?;

            if export_args.csv {
                CsvExporter::write(entries, path)?;
            }

            Ok(())
        } else {
            Err(anyhow::anyhow!("Invalid parameters"))
        }
    }
}
