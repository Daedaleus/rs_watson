use chrono::{DateTime, FixedOffset, ParseResult};
use structopt::StructOpt;

use crate::{read, write, Frame};

#[derive(StructOpt, Debug)]
#[structopt(name = "rs-watson", about = "Time-tracking in rust")]
struct Cli {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    Add(AddCommand),
}

#[derive(StructOpt, Debug)]
struct AddCommand {
    #[structopt(short, long)]
    project: String,
    #[structopt(short, long)]
    task: Option<String>,
    #[structopt(short, long, parse(try_from_str = parse_datetime))]
    from: DateTime<FixedOffset>,
    #[structopt(short, long, parse(try_from_str = parse_datetime))]
    until: DateTime<FixedOffset>,
}

fn parse_datetime(datetime: &str) -> ParseResult<DateTime<FixedOffset>> {
    DateTime::parse_from_str(datetime, "%Y-%m-%dT%H:%M:%S%z")
}

fn parse_add(cmd: AddCommand) -> anyhow::Result<()> {
    let frame = Frame {
        project: cmd.project,
        task: cmd.task,
        from: DateTime::from(cmd.from),
        until: DateTime::from(cmd.until),
    };
    write(frame, "frames.json".to_string())?;

    let frames = read("frames.json".to_string())?;
    log::debug!("{:#?}", frames);
    Ok(())
}

pub fn parse() -> anyhow::Result<()> {
    let cli = Cli::from_args();
    log::debug!("{:#?}", cli);

    match cli.cmd {
        Command::Add(add_cmd) => parse_add(add_cmd),
    }
}
