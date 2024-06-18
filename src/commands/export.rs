use chrono::NaiveDate;
use clap_derive::Args;

use crate::commands::params::{FromDate, ToDate};
use crate::commands::parse_date;
use crate::commands::{ExportArgs, Invokable};
use crate::exporter::csv::CsvExporter;
use crate::exporter::Exporter;
use crate::storage::entries::Entries;

#[derive(Args)]
pub(crate) struct Export {
    #[clap(short = 'f')]
    #[arg(value_parser(parse_date))]
    from: Option<NaiveDate>,
    #[clap(short = 't')]
    #[arg(value_parser(parse_date))]
    to: Option<NaiveDate>,
    #[clap(short = 'o')]
    path: String,
    #[command(flatten)]
    export_args: ExportArgs,
}

impl Invokable for Export {
    fn invoke(&self, entries: &mut Entries) -> anyhow::Result<()> {
        let from = FromDate::from(self.from).or_min();
        let to = ToDate::from(self.to).or_max();
        let entries = entries.get_in_range(from, to)?;

        if self.export_args.csv {
            CsvExporter::write(entries, self.path.clone())?;
        }

        Ok(())
    }
}
