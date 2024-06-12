use chrono::NaiveDate;

use crate::commands::{date_or_max, date_or_min, ExportArgs};
use crate::exporter::csv::CsvExporter;
use crate::exporter::Exporter;
use crate::storage::entries::Entries;

pub(crate) struct Export;

impl Export {
    pub fn invoke(
        entries: &mut Entries,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
        path: String,
        export_args: ExportArgs,
    ) -> anyhow::Result<()> {
        let from = date_or_min(from);
        let to = date_or_max(to);
        let entries = entries.get_in_range(from, to)?;

        if export_args.csv {
            CsvExporter::write(entries, path)?;
        }

        Ok(())
    }
}
