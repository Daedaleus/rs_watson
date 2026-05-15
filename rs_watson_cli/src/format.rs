use std::collections::BTreeMap;

use chrono::{DateTime, Duration, Local, NaiveDate, Utc};
use owo_colors::OwoColorize;
use rs_watson::{Frame, Report};

pub(crate) fn fmt_tags(tags: &[String]) -> String {
    if tags.is_empty() {
        String::new()
    } else {
        format!("  {}", format!("[{}]", tags.join(", ")).cyan())
    }
}

pub(crate) fn fmt_time(dt: DateTime<Utc>) -> String {
    dt.with_timezone(&Local).format("%H:%M:%S").to_string()
}

pub(crate) fn fmt_duration(d: Duration) -> String {
    let total = d.num_seconds().max(0);
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;

    if h > 0 {
        format!("{}h {}m {}s", h, m, s)
    } else if m > 0 {
        format!("{}m {}s", m, s)
    } else {
        format!("{}s", s)
    }
}

/// Groups frames by local date, returning a sorted map.
fn group_by_day(frames: &[Frame]) -> BTreeMap<NaiveDate, Vec<&Frame>> {
    let mut by_day: BTreeMap<NaiveDate, Vec<&Frame>> = BTreeMap::new();
    for frame in frames {
        by_day
            .entry(frame.start.with_timezone(&Local).date_naive())
            .or_default()
            .push(frame);
    }
    by_day
}

/// Prints frames as individual entries grouped by day (used by `log`).
pub(crate) fn print_frames_grouped(frames: &[Frame]) {
    for (date, day_frames) in group_by_day(frames) {
        let total = day_frames
            .iter()
            .fold(Duration::zero(), |acc, f| acc + (f.end - f.start));

        println!(
            "{}  {}",
            date.format("%A, %d %B %Y").to_string().bold().white(),
            format!("({})", fmt_duration(total)).bright_black(),
        );

        for frame in &day_frames {
            println!(
                "  {}  {}  {}   {:<12}  {}{}",
                fmt_time(frame.start).bright_white(),
                "→".white(),
                fmt_time(frame.end).bright_white(),
                fmt_duration(frame.end - frame.start).magenta().bold(),
                frame.project.yellow().bold(),
                fmt_tags(&frame.tags),
            );
        }
        println!();
    }
}

/// Prints frames as an aggregated report grouped by day (used by `today` and `report`).
/// Shows grand total when `show_total` is true and there is more than one day.
pub(crate) fn print_report_grouped(frames: &[Frame], show_total: bool) {
    let by_day = group_by_day(frames);

    let grand_total = frames
        .iter()
        .fold(Duration::zero(), |acc, f| acc + (f.end - f.start));

    for (date, day_frames) in &by_day {
        let owned: Vec<Frame> = day_frames.iter().map(|f| (*f).clone()).collect();
        let report = Report::from_frames(&owned);

        println!(
            "{}  {}",
            date.format("%A, %d %B %Y").to_string().bold().white(),
            format!("({})", fmt_duration(report.total)).bright_black(),
        );
        println!();

        for project in &report.projects {
            println!(
                "  {}  {}",
                format!("{:<20}", project.name).yellow().bold(),
                fmt_duration(project.total).magenta().bold(),
            );
            for tag in &project.tags {
                println!(
                    "    {}  {}",
                    format!("{:<18}", tag.name).cyan(),
                    fmt_duration(tag.total).magenta(),
                );
            }
        }
        println!();
    }

    if show_total && by_day.len() > 1 {
        println!(
            "{}  {}",
            "Total".bold().white(),
            fmt_duration(grand_total).magenta().bold(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt_duration_seconds_only() {
        assert_eq!(fmt_duration(Duration::seconds(45)), "45s");
    }

    #[test]
    fn fmt_duration_zero() {
        assert_eq!(fmt_duration(Duration::zero()), "0s");
    }

    #[test]
    fn fmt_duration_negative_treated_as_zero() {
        assert_eq!(fmt_duration(Duration::seconds(-10)), "0s");
    }

    #[test]
    fn fmt_duration_minutes_and_seconds() {
        assert_eq!(fmt_duration(Duration::seconds(125)), "2m 5s");
    }

    #[test]
    fn fmt_duration_hours_minutes_seconds() {
        assert_eq!(fmt_duration(Duration::seconds(3723)), "1h 2m 3s");
    }

    #[test]
    fn fmt_duration_exact_hour() {
        assert_eq!(fmt_duration(Duration::hours(2)), "2h 0m 0s");
    }
}
