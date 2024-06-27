use chrono::{NaiveDate, NaiveTime};
use clap_derive::{Args, Subcommand};
use enum_dispatch::enum_dispatch;

use crate::commands::add::Add;
use crate::commands::edit::Edit;
use crate::commands::export::Export;
use crate::commands::import::Import;
use crate::commands::log::Log;
use crate::commands::report::Report;
use crate::commands::start::Start;
use crate::commands::stop::Stop;
use crate::commands::today::Today;
use crate::storage::entries::Entries;
use crate::Args;

mod add;
pub(crate) mod edit;
pub(crate) mod export;
pub(crate) mod import;
pub(crate) mod log;
pub(crate) mod params;
pub(crate) mod report;
pub(crate) mod start;
pub(crate) mod stop;
pub(crate) mod today;

#[enum_dispatch(Invokable)]
#[derive(Subcommand)]
pub enum Command {
    #[clap(name = "start", about = "Start logging")]
    Start(Start),
    Log(Log),
    Stop(Stop),
    Report(Report),
    Today(Today),
    Export(Export),
    Edit(Edit),
    Import(Import),
    Add(Add),
}

#[derive(Args)]
#[group(required = true, multiple = false)]
pub struct ExportArgs {
    #[arg(long)]
    csv: bool,
}

#[enum_dispatch]
trait Invokable {
    fn invoke(&self, entries: &mut Entries) -> anyhow::Result<()>;
}

pub fn run(args: Args, entries: &mut Entries) -> anyhow::Result<()> {
    args.command.invoke(entries)
}

pub fn parse_time(time_str: &str) -> anyhow::Result<NaiveTime, chrono::format::ParseError> {
    NaiveTime::parse_from_str(time_str, "%H:%M:%S")
}

fn parse_date(date_str: &str) -> anyhow::Result<NaiveDate, chrono::format::ParseError> {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_time() {
        let time = parse_time("12:34:56").unwrap();
        assert_eq!(time, NaiveTime::from_hms_opt(12, 34, 56).unwrap());
    }

    #[test]
    fn test_parse_time_invalid() {
        let time = parse_time("12:34:56:78");
        assert!(time.is_err());
    }

    #[test]
    fn test_parse_date() {
        let date = parse_date("2021-01-01").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2021, 1, 1).unwrap());
    }
}
