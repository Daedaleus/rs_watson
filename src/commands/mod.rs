use chrono::{NaiveDate, NaiveTime};
use clap_derive::{Args, Subcommand};
use enum_dispatch::enum_dispatch;

use crate::commands::edit::Edit;
use crate::commands::export::Export;
use crate::commands::log::Log;
use crate::commands::report::Report;
use crate::commands::start::Start;
use crate::commands::stop::Stop;
use crate::commands::today::Today;
use crate::storage::entries::Entries;
use crate::Args;

pub(crate) mod edit;
pub(crate) mod export;
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

fn date_or_min(date: Option<NaiveDate>) -> NaiveDate {
    date.unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap())
}

fn date_or_max(date: Option<NaiveDate>) -> NaiveDate {
    date.unwrap_or_else(|| NaiveDate::from_ymd_opt(9999, 12, 31).unwrap())
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

    #[test]
    fn test_date_or_min() {
        let date = date_or_min(None);
        assert_eq!(date, NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
    }

    #[test]
    fn test_date_or_max() {
        let date = date_or_max(None);
        assert_eq!(date, NaiveDate::from_ymd_opt(9999, 12, 31).unwrap());
    }

    #[test]
    fn test_date_or_min_with_date() {
        let date = date_or_min(Some(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap()));
        assert_eq!(date, NaiveDate::from_ymd_opt(2021, 1, 1).unwrap());
    }

    #[test]
    fn test_date_or_max_with_date() {
        let date = date_or_max(Some(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap()));
        assert_eq!(date, NaiveDate::from_ymd_opt(2021, 1, 1).unwrap());
    }
}
