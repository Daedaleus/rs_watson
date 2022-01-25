use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct BaseCli {
    #[structopt(short, long)]
    pub project: String,
    #[structopt(short, long)]
    pub task: Option<String>,
    #[structopt(short, long, parse(try_from_str = parse_datetime))]
    pub from: Option<DateTime<Local>>,
}

fn parse_datetime(datetime: &str) -> Option<DateTime<Local>> {
    let naiv = NaiveDateTime::parse_from_str(datetime, "%Y-%m-%dT%H:%M:%S%z")?;
    let datetime = Local.from_local_datetime(&naiv);
    match datetime.unwrap() {
        Ok(a) => Some(a),
        _ => None,
    }
}

pub fn get_args() -> BaseCli {
    BaseCli::from_args()
}