use chrono::NaiveDate;
use clap_derive::Args;

use crate::commands::params::{FromDate, Project, ToDate};
use crate::commands::{parse_date, Invokable};
use crate::storage::entries::Entries;

#[derive(Args)]
pub(crate) struct Report {
    #[arg(short = 'f')]
    #[arg(value_parser(parse_date))]
    from: Option<NaiveDate>,
    #[arg(short = 't')]
    #[arg(value_parser(parse_date))]
    to: Option<NaiveDate>,
    #[arg(short = 'p')]
    project: Option<Project>,
}

impl Invokable for Report {
    fn invoke(&self, entries: &mut Entries) -> anyhow::Result<()> {
        let from = FromDate::from(self.from).or_min();
        let to = ToDate::from(self.to).or_max();
        let time_framed_entries = entries.get_in_range(from, to)?;
        let entries = match self.project.clone() {
            Some(project) => time_framed_entries.filter_by_project(project),
            None => time_framed_entries,
        };
        entries.report();
        Ok(())
    }
}

impl Report {
    pub fn new(from: Option<NaiveDate>, to: Option<NaiveDate>, project: Option<Project>) -> Self {
        Self { from, to, project }
    }
}
