use chrono::{DateTime, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use rs_watson::Frame;

pub(crate) fn fmt_duration(d: Duration) -> String {
    let total = d.num_seconds().max(0);
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{h}h {m:02}m {s:02}s")
    } else if m > 0 {
        format!("{m}m {s:02}s")
    } else {
        format!("{s}s")
    }
}

pub(crate) fn fmt_time(dt: DateTime<Utc>) -> String {
    dt.with_timezone(&Local).format("%H:%M:%S").to_string()
}

pub(crate) fn fmt_local_dt(dt: DateTime<Utc>) -> String {
    dt.with_timezone(&Local)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

pub(crate) fn fmt_tags(tags: &[String]) -> String {
    if tags.is_empty() {
        String::new()
    } else {
        format!("  [{}]", tags.join(", "))
    }
}

pub(crate) fn parse_local_dt(s: &str) -> Option<DateTime<Utc>> {
    let s = s.trim();
    let try_naive = |fmt| NaiveDateTime::parse_from_str(s, fmt).ok();
    let naive = try_naive("%Y-%m-%d %H:%M:%S")
        .or_else(|| try_naive("%Y-%m-%d %H:%M"))
        .or_else(|| {
            NaiveTime::parse_from_str(s, "%H:%M:%S")
                .or_else(|_| NaiveTime::parse_from_str(s, "%H:%M"))
                .ok()
                .map(|t| Local::now().date_naive().and_time(t))
        })?;
    Local
        .from_local_datetime(&naive)
        .single()
        .map(|dt| dt.with_timezone(&Utc))
}

pub(crate) fn parse_local_date(s: &str) -> Option<NaiveDate> {
    let s = s.trim().to_lowercase();
    match s.as_str() {
        "" => None,
        "today" => Some(Local::now().date_naive()),
        "yesterday" => Some(Local::now().date_naive() - Duration::days(1)),
        _ => NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok(),
    }
}

pub(crate) fn collect_projects(frames: &[Frame]) -> Vec<String> {
    let mut names: Vec<String> = frames.iter().map(|f| f.project.clone()).collect();
    names.sort();
    names.dedup();
    names
}

pub(crate) fn parse_tags(s: &str) -> Vec<String> {
    s.split(',')
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect()
}
