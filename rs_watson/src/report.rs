use std::collections::HashMap;

use chrono::Duration;

use crate::Frame;

pub struct Report {
    pub total: Duration,
    pub projects: Vec<ProjectReport>,
}

pub struct ProjectReport {
    pub name: String,
    pub total: Duration,
    pub tags: Vec<TagReport>,
}

pub struct TagReport {
    pub name: String,
    pub total: Duration,
}

impl Report {
    pub fn from_frames(frames: &[Frame]) -> Self {
        let mut project_map: HashMap<String, (Duration, HashMap<String, Duration>)> =
            HashMap::new();

        for frame in frames {
            let duration = frame.end - frame.start;
            let entry = project_map
                .entry(frame.project.clone())
                .or_insert_with(|| (Duration::zero(), HashMap::new()));
            entry.0 = entry.0 + duration;
            for tag in &frame.tags {
                let tag_total = entry.1.entry(tag.clone()).or_insert(Duration::zero());
                *tag_total = *tag_total + duration;
            }
        }

        let mut projects: Vec<ProjectReport> = project_map
            .into_iter()
            .map(|(name, (total, tags))| {
                let mut tag_list: Vec<TagReport> = tags
                    .into_iter()
                    .map(|(name, total)| TagReport { name, total })
                    .collect();
                // sort tags by total descending, then alphabetically
                tag_list.sort_by(|a, b| b.total.cmp(&a.total).then(a.name.cmp(&b.name)));
                ProjectReport { name, total, tags: tag_list }
            })
            .collect();
        // sort projects by total descending, then alphabetically
        projects.sort_by(|a, b| b.total.cmp(&a.total).then(a.name.cmp(&b.name)));

        let total = projects.iter().fold(Duration::zero(), |acc, p| acc + p.total);

        Report { total, projects }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;
    use crate::Frame;

    fn t(h: u32) -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 15, h, 0, 0).unwrap()
    }

    fn frame(project: &str, tags: &[&str], start_h: u32, end_h: u32) -> Frame {
        Frame {
            id: Uuid::new_v4(),
            project: project.into(),
            tags: tags.iter().map(|s| s.to_string()).collect(),
            start: t(start_h),
            end: t(end_h),
        }
    }

    #[test]
    fn empty_frames_produce_empty_report() {
        let r = Report::from_frames(&[]);
        assert!(r.projects.is_empty());
        assert_eq!(r.total, Duration::zero());
    }

    #[test]
    fn single_frame_no_tags() {
        let r = Report::from_frames(&[frame("backend", &[], 9, 11)]);
        assert_eq!(r.projects.len(), 1);
        assert_eq!(r.projects[0].name, "backend");
        assert_eq!(r.projects[0].total, Duration::hours(2));
        assert!(r.projects[0].tags.is_empty());
        assert_eq!(r.total, Duration::hours(2));
    }

    #[test]
    fn multiple_frames_same_project_totals_sum() {
        let r = Report::from_frames(&[
            frame("backend", &[], 9, 10),
            frame("backend", &[], 11, 13),
        ]);
        assert_eq!(r.projects.len(), 1);
        assert_eq!(r.projects[0].total, Duration::hours(3));
    }

    #[test]
    fn projects_sorted_by_total_descending() {
        let r = Report::from_frames(&[
            frame("small",  &[], 9, 10),
            frame("big",    &[], 9, 13),
            frame("medium", &[], 9, 11),
        ]);
        assert_eq!(r.projects[0].name, "big");
        assert_eq!(r.projects[1].name, "medium");
        assert_eq!(r.projects[2].name, "small");
    }

    #[test]
    fn tags_get_full_frame_duration_each() {
        let r = Report::from_frames(&[frame("backend", &["api", "auth"], 9, 11)]);
        let proj = &r.projects[0];
        let api  = proj.tags.iter().find(|t| t.name == "api").unwrap();
        let auth = proj.tags.iter().find(|t| t.name == "auth").unwrap();
        assert_eq!(api.total,  Duration::hours(2));
        assert_eq!(auth.total, Duration::hours(2));
    }

    #[test]
    fn tag_totals_accumulate_across_frames() {
        let r = Report::from_frames(&[
            frame("backend", &["api"],        9, 11), // api: 2h
            frame("backend", &["api", "auth"], 11, 12), // api: +1h, auth: 1h
        ]);
        let proj = &r.projects[0];
        let api  = proj.tags.iter().find(|t| t.name == "api").unwrap();
        let auth = proj.tags.iter().find(|t| t.name == "auth").unwrap();
        assert_eq!(api.total,  Duration::hours(3));
        assert_eq!(auth.total, Duration::hours(1));
    }

    #[test]
    fn tags_sorted_by_total_descending() {
        let r = Report::from_frames(&[
            frame("backend", &["big"],        9, 12), // big: 3h
            frame("backend", &["big", "small"], 12, 13), // big: +1h, small: 1h
        ]);
        let proj = &r.projects[0];
        assert_eq!(proj.tags[0].name, "big");
        assert_eq!(proj.tags[1].name, "small");
    }

    #[test]
    fn grand_total_equals_sum_of_project_totals() {
        let r = Report::from_frames(&[
            frame("a", &[], 9, 10),
            frame("b", &[], 10, 12),
        ]);
        assert_eq!(r.total, Duration::hours(3));
    }
}
