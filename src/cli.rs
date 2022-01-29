use chrono::{DateTime, FixedOffset, ParseResult};
use structopt::StructOpt;

use crate::storing_engines::json::write_all;
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
    Start(StartCommand),
    Stop(StopCommand),
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

#[derive(StructOpt, Debug)]
struct StartCommand {
    #[structopt(short, long)]
    project: String,
    #[structopt(short, long)]
    task: Option<String>,
}

#[derive(StructOpt, Debug)]
struct StopCommand {}

fn parse_datetime(datetime: &str) -> ParseResult<DateTime<FixedOffset>> {
    DateTime::parse_from_str(datetime, "%Y-%m-%dT%H:%M:%S%z")
}

pub fn parse() -> anyhow::Result<()> {
    let cli = Cli::from_args();
    log::debug!("{:#?}", cli);

    match cli.cmd {
        Command::Add(add_cmd) => parse_add(add_cmd),
        Command::Start(start_cmd) => parse_start(start_cmd),
        Command::Stop(stop_cmd) => parse_stop(stop_cmd),
    }
}

fn parse_add(cmd: AddCommand) -> anyhow::Result<()> {
    let frame = Frame {
        project: cmd.project,
        task: cmd.task,
        from: DateTime::from(cmd.from),
        until: Some(DateTime::from(cmd.until)),
    };
    write(frame, "frames.json".to_string())?;

    let frames = read("frames.json".to_string())?;
    log::debug!("{:#?}", frames);
    Ok(())
}

fn parse_start(cmd: StartCommand) -> anyhow::Result<()> {
    let frame = Frame {
        project: cmd.project,
        task: cmd.task,
        from: chrono::offset::Utc::now(),
        until: None,
    };
    log::debug!("{:#?}", frame);
    write(frame, "frames.json".to_string())?;
    let frames = read("frames.json".to_string())?;
    log::debug!("{:#?}", frames);
    Ok(())
}

fn parse_stop(_: StopCommand) -> anyhow::Result<()> {
    let mut frames = read("frames.json".to_string())?;
    let mut frame = frames.first_mut().expect("Not found!");
    frame.until = Some(chrono::offset::Utc::now());
    write_all(frames, "frames.json".to_string())?;
    let frames = read("frames.json".to_string())?;
    log::debug!("{:#?}", frames);
    Ok(())
}
