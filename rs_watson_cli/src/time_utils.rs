use anyhow::{Context, Result};
use chrono::{
    DateTime, Datelike, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc,
};
use dialoguer::{Input, theme::ColorfulTheme};
use owo_colors::OwoColorize;

use rs_watson::config::{Config, WeekStart};

/// Parses a date string into a `NaiveDate` in local time.
/// Accepts `YYYY-MM-DD` or shortcuts: `today`, `yesterday`, `week`, `month`.
/// The `week` shortcut respects `week_start` (Monday or Sunday).
pub(crate) fn parse_date(input: &str, week_start: WeekStart) -> Result<NaiveDate> {
    let today = Local::now().date_naive();
    match input.trim().to_lowercase().as_str() {
        "today" => Ok(today),
        "yesterday" => Ok(today - Duration::days(1)),
        "week" => {
            let days_back = match week_start {
                WeekStart::Monday => today.weekday().num_days_from_monday() as i64,
                WeekStart::Sunday => today.weekday().num_days_from_sunday() as i64,
            };
            Ok(today - Duration::days(days_back))
        }
        "month" => Ok(today.with_day(1).expect("day 1 always valid")),
        s => NaiveDate::parse_from_str(s, "%Y-%m-%d").with_context(|| {
            format!("Invalid date \"{s}\", expected YYYY-MM-DD or: today, yesterday, week, month")
        }),
    }
}

/// Parses a local datetime string and returns UTC. Accepted formats:
/// - `HH:MM` / `HH:MM:SS`  → today's date
/// - `YYYY-MM-DD HH:MM` / `YYYY-MM-DD HH:MM:SS`  → explicit date
/// - `today HH:MM[:SS]` / `yesterday HH:MM[:SS]`  → relative date shortcuts
pub(crate) fn parse_at(input: &str) -> Result<DateTime<Utc>> {
    let s = input.trim();

    // YYYY-MM-DD HH:MM:SS
    if let Ok(naive) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return local_naive_to_utc(naive);
    }
    // YYYY-MM-DD HH:MM
    if let Ok(naive) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M") {
        return local_naive_to_utc(naive);
    }

    // "today HH:MM[:SS]" / "yesterday HH:MM[:SS]"
    if let Some(dt) = parse_relative_datetime(s) {
        return Ok(dt);
    }

    // HH:MM:SS or HH:MM → today
    let time = NaiveTime::parse_from_str(s, "%H:%M:%S")
        .or_else(|_| NaiveTime::parse_from_str(s, "%H:%M"))
        .with_context(|| format!(
            "Invalid time \"{s}\". Use HH:MM, HH:MM:SS, YYYY-MM-DD HH:MM[:SS], or yesterday/today HH:MM[:SS]"
        ))?;
    local_naive_to_utc(Local::now().date_naive().and_time(time))
}

fn local_naive_to_utc(naive: NaiveDateTime) -> Result<DateTime<Utc>> {
    Local
        .from_local_datetime(&naive)
        .single()
        .map(|dt| dt.with_timezone(&Utc))
        .context("Ambiguous datetime (DST transition)")
}

fn parse_relative_datetime(input: &str) -> Option<DateTime<Utc>> {
    let (word, time_str) = input.split_once(' ')?;

    let today = Local::now().date_naive();
    let date = match word.to_lowercase().as_str() {
        "today" => today,
        "yesterday" => today - Duration::days(1),
        _ => return None,
    };

    let time = NaiveTime::parse_from_str(time_str.trim(), "%H:%M:%S")
        .or_else(|_| NaiveTime::parse_from_str(time_str.trim(), "%H:%M"))
        .ok()?;

    // DST ambiguity is silently ignored — caller falls through to other formats.
    local_naive_to_utc(date.and_time(time)).ok()
}

/// Prompts for a datetime value in local time. Pre-filled with the full date+time of `default`.
///
/// Accepted inputs:
/// - `HH:MM[:SS]`  → keeps the original date, changes only the time
/// - `YYYY-MM-DD HH:MM[:SS]`  → fully replaces date and time
/// - `yesterday HH:MM[:SS]` / `today HH:MM[:SS]`  → relative date shortcuts
pub(crate) fn prompt_time(prompt: &str, default: DateTime<Utc>) -> Result<DateTime<Utc>> {
    let local_default = default.with_timezone(&Local);
    loop {
        let input = Input::<String>::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .with_initial_text(local_default.format("%Y-%m-%d %H:%M:%S").to_string())
            .interact_text()?;

        let s = input.trim();

        // Try all absolute/relative datetime formats first.
        if let Ok(dt) = parse_at(s) {
            return Ok(dt);
        }

        // Fall back: time-only on the original frame's date (not today).
        let parsed = NaiveTime::parse_from_str(s, "%H:%M:%S")
            .or_else(|_| NaiveTime::parse_from_str(s, "%H:%M"));

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
            Err(_) => eprintln!(
                "  {} Use HH:MM, YYYY-MM-DD HH:MM, or yesterday/today HH:MM",
                "Invalid.".red()
            ),
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
    use rs_watson::config::{BehaviorConfig, LogConfig, StorageConfig, WeekStart};

    fn cfg(allow_future: bool) -> Config {
        Config {
            storage: StorageConfig {
                provider: Default::default(),
                data_dir: None,
            },
            behavior: BehaviorConfig {
                allow_future_times: allow_future,
                week_start: WeekStart::Monday,
            },
            log: LogConfig::default(),
            epics: vec![],
        }
    }

    // --- parse_date ---

    #[test]
    fn parse_date_accepts_iso_format() {
        let d = parse_date("2026-05-15", WeekStart::Monday).unwrap();
        assert_eq!(d, NaiveDate::from_ymd_opt(2026, 5, 15).unwrap());
    }

    #[test]
    fn parse_date_today_equals_local_today() {
        assert_eq!(
            parse_date("today", WeekStart::Monday).unwrap(),
            Local::now().date_naive()
        );
    }

    #[test]
    fn parse_date_yesterday_is_one_day_before_today() {
        let expected = Local::now().date_naive() - Duration::days(1);
        assert_eq!(
            parse_date("yesterday", WeekStart::Monday).unwrap(),
            expected
        );
    }

    #[test]
    fn parse_date_week_monday_start() {
        let today = Local::now().date_naive();
        let expected = today - Duration::days(today.weekday().num_days_from_monday() as i64);
        assert_eq!(parse_date("week", WeekStart::Monday).unwrap(), expected);
    }

    #[test]
    fn parse_date_week_sunday_start() {
        let today = Local::now().date_naive();
        let expected = today - Duration::days(today.weekday().num_days_from_sunday() as i64);
        assert_eq!(parse_date("week", WeekStart::Sunday).unwrap(), expected);
    }

    #[test]
    fn parse_date_month_is_first_of_current_month() {
        let today = Local::now().date_naive();
        let expected = today.with_day(1).unwrap();
        assert_eq!(parse_date("month", WeekStart::Monday).unwrap(), expected);
    }

    #[test]
    fn parse_date_rejects_invalid() {
        assert!(parse_date("invalid", WeekStart::Monday).is_err());
        assert!(parse_date("2026/05/15", WeekStart::Monday).is_err());
        assert!(parse_date("", WeekStart::Monday).is_err());
    }

    // --- parse_at ---

    #[test]
    fn parse_at_accepts_full_datetime() {
        let dt = parse_at("2026-05-14 09:00").unwrap();
        let local = dt.with_timezone(&Local);
        assert_eq!(
            local.format("%Y-%m-%d %H:%M").to_string(),
            "2026-05-14 09:00"
        );
    }

    #[test]
    fn parse_at_accepts_full_datetime_with_seconds() {
        let dt = parse_at("2026-05-14 09:30:15").unwrap();
        let local = dt.with_timezone(&Local);
        assert_eq!(
            local.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2026-05-14 09:30:15"
        );
    }

    #[test]
    fn parse_at_accepts_yesterday_shortcut() {
        let expected_date = (Local::now() - Duration::days(1)).date_naive();
        let dt = parse_at("yesterday 09:00").unwrap();
        assert_eq!(dt.with_timezone(&Local).date_naive(), expected_date);
    }

    #[test]
    fn parse_at_accepts_today_shortcut() {
        let expected_date = Local::now().date_naive();
        let dt = parse_at("today 10:30").unwrap();
        assert_eq!(dt.with_timezone(&Local).date_naive(), expected_date);
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
