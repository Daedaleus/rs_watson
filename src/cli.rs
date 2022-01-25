use chrono::{DateTime, FixedOffset, ParseResult};
use structopt::StructOpt;

use crate::{read, write, Frame};

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct BaseCli {
    #[structopt(short, long)]
    pub project: String,
    #[structopt(short, long)]
    pub task: Option<String>,
    #[structopt(short, long, parse(try_from_str = parse_datetime))]
    pub from: DateTime<FixedOffset>,
    #[structopt(short, long, parse(try_from_str = parse_datetime))]
    pub until: DateTime<FixedOffset>,
}

fn parse_datetime(datetime: &str) -> ParseResult<DateTime<FixedOffset>> {
    DateTime::parse_from_str(datetime, "%Y-%m-%dT%H:%M:%S%z")
}

pub fn parse() -> anyhow::Result<()> {
    let args = BaseCli::from_args();
    log::debug!("{:#?}", args);

    let frame = Frame {
        project: args.project,
        task: args.task,
        from: DateTime::from(args.from),
        until: DateTime::from(args.from),
    };
    write(frame, "frames.json".to_string())?;

    let frames = read("frames.json".to_string())?;
    log::debug!("{:#?}", frames);
    Ok(())
}
