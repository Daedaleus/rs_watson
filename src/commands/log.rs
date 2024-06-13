use chrono::NaiveDate;
use clap_derive::Args;

use crate::commands::{date_or_max, date_or_min, parse_date, Invokable};
use crate::storage::entries::Entries;

#[derive(Args)]
pub struct Log {
    #[clap(short = 'f')]
    #[arg(value_parser(parse_date))]
    from: Option<NaiveDate>,
    #[clap(short = 't')]
    #[arg(value_parser(parse_date))]
    to: Option<NaiveDate>,
}

impl Invokable for Log {
    fn invoke(&self, entries: &mut Entries) -> anyhow::Result<()> {
        let from = date_or_min(self.from);
        let to = date_or_max(self.to);
        let entries = entries.get_in_range(from, to)?;
        println!("{}", entries);
        Ok(())
    }
}
