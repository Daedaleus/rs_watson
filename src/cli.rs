use chrono::{Datelike, DateTime, Local, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use regex::Regex;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct BaseCli {
    #[structopt(short, long)]
    pub project: String,
    #[structopt(short, long)]
    pub task: Option<String>,
    #[structopt(short, long)]
    pub from: String,
}

impl BaseCli {
    pub fn datetime_from_string(&self) -> anyhow::Result<NaiveDateTime> {
        let string = self.from.clone();
        
        let time_regexp = Regex::new(r"^\d{2}:\d{2}$")?;
        let datetime_regexp = Regex::new(r"^\d{2}:\d{2} \d{2}.\d{2}.\d{4}$")?;

        let today = chrono::Local::today();

        if time_regexp.is_match(&string) {
            let date = NaiveDate::from_ymd(today.year(), today.month(), today.day());
            let time = NaiveTime::from_hms(time_regexp.captures(&string)[1], time_regexp.captures(&string)[2], 0);
            anyhow::Ok(NaiveDateTime::new(date, time))
        } else if datetime_regexp.is_match(&string) {
            let date = NaiveDate::from_ymd(datetime_regexp.captures(&string)[5], datetime_regexp.captures(&string)[4], datetime_regexp.captures(&string)[3]);
            let time = NaiveTime::from_hms(datetime_regexp.captures(&string)[1], datetime_regexp.captures(&string)[2], 0);
            anyhow::Ok(NaiveDateTime(date, time))
        } else {
            anyhow::Error::new("Bad time format")
        }
    }
}

pub fn get_args() -> BaseCli {
    BaseCli::from_args()
}