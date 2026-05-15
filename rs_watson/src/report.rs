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
