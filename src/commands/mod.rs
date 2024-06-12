use chrono::{NaiveDate, NaiveTime};
use clap_derive::{Args, Subcommand};

use crate::Args;
use crate::commands::Command::{Edit, Export, Log, Report, Start, Stop, Today};
use crate::storage::entries::Entries;

pub(crate) mod edit;
pub(crate) mod export;
pub(crate) mod log;
pub(crate) mod report;
pub(crate) mod start;
pub(crate) mod stop;

#[derive(Subcommand)]
pub enum Command {
    #[clap(name = "start", about = "Start logging")]
    Start {
        project: String,
        tags: Option<Vec<String>>,
        #[clap(short = 'a')]
        #[arg(value_parser(parse_time))]
        at: Option<NaiveTime>,
    },
    Log {
        #[clap(short = 'f')]
        #[arg(value_parser(parse_date))]
        from: Option<NaiveDate>,
        #[clap(short = 't')]
        #[arg(value_parser(parse_date))]
        to: Option<NaiveDate>,
    },
    Stop {
        #[clap(short = 'a')]
        #[arg(value_parser(parse_time))]
        at: Option<NaiveTime>,
    },
    Report {
        #[clap(short = 'f')]
        #[arg(value_parser(parse_date))]
        from: Option<NaiveDate>,
        #[clap(short = 't')]
        #[arg(value_parser(parse_date))]
        to: Option<NaiveDate>,
        #[clap(short = 'p')]
        project: Option<String>,
    },
    Today,
    Export {
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
    },
    Edit {
        hash: String,
    },
}

#[derive(Args)]
#[group(required = true, multiple = false)]
pub struct ExportArgs {
    #[arg(long)]
    csv: bool,
}

struct Invoker;

impl Invoker {
    pub fn invoke<C: Invokable>(
        command: C,
        entries: &mut Entries,
        params: Command,
    ) -> anyhow::Result<()> {
        command.invoke(entries, params)
    }
}

trait Invokable {
    fn invoke(&self, entries: &mut Entries, params: Command) -> anyhow::Result<()>;
}

pub fn run(args: Args, entries: &mut Entries) -> anyhow::Result<()> {
    match args.command {
        Start { project, tags, at } => {
            Invoker::invoke(start::Start, entries, Start { project, tags, at })
        }
        Stop { at } => Invoker::invoke(stop::Stop, entries, Stop { at }),
        Log { from, to } => Invoker::invoke(log::Log, entries, Log { from, to }),
        Report { from, to, project } => {
            Invoker::invoke(report::Report, entries, Report { from, to, project })
        }
        Export {
            from,
            to,
            path,
            export_args,
        } => Invoker::invoke(
            export::Export,
            entries,
            Export {
                from,
                to,
                path,
                export_args,
            },
        ),
        Edit { hash } => Invoker::invoke(edit::Edit, entries, Edit { hash }),
        Today => {
            let today = chrono::Local::now().naive_local().date();
            Invoker::invoke(
                report::Report,
                entries,
                Report {
                    from: Some(today),
                    to: Some(today),
                    project: None,
                },
            )
        }
    }
}

fn parse_time(time_str: &str) -> anyhow::Result<NaiveTime, chrono::format::ParseError> {
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
