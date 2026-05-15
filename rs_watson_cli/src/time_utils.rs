use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, NaiveTime, TimeZone, Utc};
use dialoguer::{Input, theme::ColorfulTheme};
use owo_colors::OwoColorize;

use crate::config::Config;

/// Parses a date string into a `NaiveDate` in local time.
/// Accepts `YYYY-MM-DD` or shortcuts: `today`, `yesterday`, `week` (start of current week),
/// `month` (start of current month).
pub(crate) fn parse_date(input: &str) -> Result<NaiveDate> {
    let today = Local::now().date_naive();
    match input.trim().to_lowercase().as_str() {
        "today"     => Ok(today),
        "yesterday" => Ok(today - Duration::days(1)),
        "week"      => Ok(today - Duration::days(today.weekday().num_days_from_monday() as i64)),
        "month"     => Ok(today.with_day(1).expect("day 1 always valid")),
        s => NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .with_context(|| format!(
                "Invalid date \"{s}\", expected YYYY-MM-DD or: today, yesterday, week, month"
            )),
    }
}

/// Parses a local time string (HH:MM or HH:MM:SS) relative to today and returns UTC.
pub(crate) fn parse_at(input: &str) -> Result<DateTime<Utc>> {
    let local_now = Local::now();
    let time = NaiveTime::parse_from_str(input.trim(), "%H:%M:%S")
        .or_else(|_| NaiveTime::parse_from_str(input.trim(), "%H:%M"))
        .with_context(|| format!("Invalid time \"{input}\", expected HH:MM or HH:MM:SS"))?;
    let naive_local = local_now.date_naive().and_time(time);
    Local
        .from_local_datetime(&naive_local)
        .single()
        .map(|dt| dt.with_timezone(&Utc))
        .context("Ambiguous time (DST transition)")
}

/// Prompts for a time value in local time, pre-filled with the local representation of `default`.
/// Parses the input as local time on the same local date, then converts back to UTC.
pub(crate) fn prompt_time(prompt: &str, default: DateTime<Utc>) -> Result<DateTime<Utc>> {
    let local_default = default.with_timezone(&Local);
    loop {
        let input = Input::<String>::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .with_initial_text(local_default.format("%H:%M:%S").to_string())
            .interact_text()?;

        let parsed = NaiveTime::parse_from_str(input.trim(), "%H:%M:%S")
            .or_else(|_| NaiveTime::parse_from_str(input.trim(), "%H:%M"));

        match parsed {
            Ok(t) => {
                let naive_local = local_default.date_naive().and_time(t);
                match Local.from_local_datetime(&naive_local).single() {
                    Some(local_dt) => return Ok(local_dt.with_timezone(&Utc)),
                    None => eprintln!(
                        "  {} Ambiguous time (DST transition), try again.",
                        "Warning:".yellow()
                    ),
                }
            }
            Err(_) => eprintln!("  {} Use HH:MM or HH:MM:SS", "Invalid time.".red()),
        }
    }
}

pub(crate) fn check_future(dt: DateTime<Utc>, config: &Config) -> Result<()> {
    if !config.behavior.allow_future_times && dt > Utc::now() {
        anyhow::bail!(
            "Future times are not allowed. Set allow_future_times = true in config to enable."
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use crate::config::{BehaviorConfig, StorageConfig, StorageProvider};

    fn cfg(allow_future: bool) -> Config {
        Config {
            storage: StorageConfig { provider: StorageProvider::Json },
            behavior: BehaviorConfig { allow_future_times: allow_future },
        }
    }

    #[test]
    fn parse_at_rejects_invalid_format() {
        assert!(parse_at("25:00").is_err());
        assert!(parse_at("abc").is_err());
        assert!(parse_at("").is_err());
    }

    #[test]
    fn parse_at_accepts_hhmm() {
        assert!(parse_at("08:30").is_ok());
    }

    #[test]
    fn parse_at_accepts_hhmmss() {
        assert!(parse_at("08:30:00").is_ok());
    }

    #[test]
    fn check_future_past_time_always_ok() {
        let past = Utc::now() - Duration::hours(1);
        assert!(check_future(past, &cfg(false)).is_ok());
        assert!(check_future(past, &cfg(true)).is_ok());
    }

    #[test]
    fn check_future_future_time_rejected_by_default() {
        let future = Utc::now() + Duration::hours(1);
        assert!(check_future(future, &cfg(false)).is_err());
    }

    #[test]
    fn check_future_future_time_allowed_when_configured() {
        let future = Utc::now() + Duration::hours(1);
        assert!(check_future(future, &cfg(true)).is_ok());
    }
}
