use chrono::Duration;
use owo_colors::OwoColorize;
use rs_watson::config::EpicConfig;
use rs_watson::{Frame, Report, resolve_epic};

use crate::format::{fmt_duration, print_project_breakdown};

/// Prints an aggregated report grouped by epic, then project, then tag.
/// Frames with no matching epic are shown under "Unassigned".
pub(crate) fn print_epic_report(frames: &[Frame], epics: &[EpicConfig], show_total: bool) {
    // Build buckets in config order so the display matches the user's ordering.
    let mut buckets: Vec<(&str, Vec<&Frame>)> =
        epics.iter().map(|e| (e.name.as_str(), vec![])).collect();
    let mut unassigned: Vec<&Frame> = vec![];

    for frame in frames {
        match resolve_epic(frame, epics) {
            Some(name) => {
                if let Some(b) = buckets.iter_mut().find(|(n, _)| *n == name) {
                    b.1.push(frame);
                }
            }
            None => unassigned.push(frame),
        }
    }

    let grand_total = frames
        .iter()
        .fold(Duration::zero(), |acc, f| acc + (f.end - f.start));

    for (name, epic_frames) in buckets.iter().filter(|(_, f)| !f.is_empty()) {
        let owned: Vec<Frame> = epic_frames.iter().copied().cloned().collect();
        let report = Report::from_frames(&owned);
        println!(
            "{}  {}",
            format!("◆  {name}").cyan().bold(),
            format!("({})", fmt_duration(report.total)).bright_black(),
        );
        println!();
        print_project_breakdown(&report);
        println!();
    }

    if !unassigned.is_empty() {
        let owned: Vec<Frame> = unassigned.iter().copied().cloned().collect();
        let report = Report::from_frames(&owned);
        println!(
            "{}  {}",
            "◆  Unassigned".bright_black().bold(),
            format!("({})", fmt_duration(report.total)).bright_black(),
        );
        println!();
        print_project_breakdown(&report);
        println!();
    }

    if show_total {
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
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    fn frame(project: &str, tags: &[&str]) -> Frame {
        Frame {
            id: Uuid::new_v4(),
            project: project.into(),
            tags: tags.iter().map(|s| s.to_string()).collect(),
            start: Utc.with_ymd_and_hms(2026, 1, 15, 9, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap(),
        }
    }

    fn epic(name: &str, project: &str, tags: &[&str]) -> EpicConfig {
        EpicConfig {
            name: name.into(),
            project: project.into(),
            tags: tags.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn resolves_project_match_without_tags() {
        let epics = vec![epic("Backend", "backend", &[])];
        assert_eq!(
            resolve_epic(&frame("backend", &["api"]), &epics),
            Some("Backend")
        );
    }

    #[test]
    fn no_match_for_different_project() {
        let epics = vec![epic("Backend", "backend", &[])];
        assert_eq!(resolve_epic(&frame("frontend", &[]), &epics), None);
    }

    #[test]
    fn tag_filter_must_be_subset_of_frame_tags() {
        let epics = vec![epic("Refactor", "backend", &["refactor"])];
        assert_eq!(
            resolve_epic(&frame("backend", &["refactor", "api"]), &epics),
            Some("Refactor")
        );
        assert_eq!(resolve_epic(&frame("backend", &["api"]), &epics), None);
    }

    #[test]
    fn most_specific_epic_wins() {
        let epics = vec![
            epic("Backend General", "backend", &[]),
            epic("Backend Refactor", "backend", &["refactor"]),
        ];
        assert_eq!(
            resolve_epic(&frame("backend", &["refactor"]), &epics),
            Some("Backend Refactor")
        );
        assert_eq!(
            resolve_epic(&frame("backend", &["api"]), &epics),
            Some("Backend General")
        );
    }

    #[test]
    fn returns_none_with_no_epics_configured() {
        assert_eq!(resolve_epic(&frame("backend", &[]), &[]), None);
    }
}
