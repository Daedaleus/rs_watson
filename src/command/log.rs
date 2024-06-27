use chrono::NaiveDate;
use clap_derive::Args;

use crate::command::param::from_date::FromDate;
use crate::command::param::to_date::ToDate;
use crate::command::utils::time::parse_date;
use crate::command::Invokable;
use crate::storage::entries::Entries;

#[derive(Args)]
pub struct Log {
    #[arg(short = 'f')]
    #[arg(value_parser(parse_date))]
    from: Option<NaiveDate>,
    #[arg(short = 't')]
    #[arg(value_parser(parse_date))]
    to: Option<NaiveDate>,
}

impl Invokable for Log {
    fn invoke(&self, entries: &mut Entries) -> anyhow::Result<()> {
        let from = FromDate::from(self.from).or_min();
        let to = ToDate::from(self.to).or_max();
        let entries = entries.get_in_range(from, to)?;
        println!("{}", entries);
        Ok(())
    }
}
