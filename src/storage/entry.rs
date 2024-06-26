use std::fmt::Display;

use anyhow::Context;
use chrono::{DateTime, TimeZone, Utc};
use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::commands::params::{Project, Tags};
use crate::importer::ts_watson::TdWatsonFrame;
use crate::storage::gen_id;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Entry {
    id: String,
    project: Project,
    tags: Option<Tags>,
    start: DateTime<Utc>,
    #[serde(default)]
    end: Option<DateTime<Utc>>,
}

impl Entry {
    pub fn new(
        project: Project,
        tags: Option<Tags>,
        start: DateTime<Utc>,
        end: Option<DateTime<Utc>>,
    ) -> anyhow::Result<Self> {
        let id = gen_id()?;
        Ok(Self {
            id,
            project,
            tags,
            start,
            end,
        })
    }

    pub fn print_report(&self) -> anyhow::Result<String> {
        let tags = self.tags.clone().unwrap_or_default().as_string();
        let duration = self.get_duration_as_human_readable()?;
        Ok(format!("[{}   {}]", tags.bright_cyan(), duration.cyan()))
    }

    fn calculate_duration(&self) -> anyhow::Result<i64> {
        let end = self.end.context("Entry has no end time")?;
        Ok((end - self.start).num_seconds())
    }

    fn get_duration_as_human_readable(&self) -> anyhow::Result<String> {
        let duration = self.calculate_duration()?;
        let hours = duration / 3600;
        let minutes = (duration % 3600) / 60;
        Ok(format!("{:02}:{:02}", hours, minutes))
    }

    pub fn get_project(&self) -> Project {
        self.project.clone()
    }

    pub fn get_tags(&self) -> Option<Tags> {
        self.tags.clone()
    }

    pub fn get_start(&self) -> DateTime<Utc> {
        self.start
    }

    pub fn set_end(&mut self, end: DateTime<Utc>) {
        self.end = Some(end);
    }

    pub fn is_running(&self) -> bool {
        self.end.is_none()
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }
}

impl Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tags = self.tags.clone().unwrap_or_default().as_string();
        let id = &self.id[..8];
        let end = match self.end {
            Some(end) => end.format("%H:%M").to_string(),
            None => "now".to_string(),
        };
        write!(
            f,
            "{}  {} to {}  {}    [{}]",
            id,
            self.start.format("%H:%M").to_string().cyan(),
            end.cyan(),
            self.project.bright_cyan(),
            tags.bright_cyan()
        )
    }
}

impl From<TdWatsonFrame> for Entry {
    fn from(imported_frame: TdWatsonFrame) -> Self {
        let project = Project::new(imported_frame.project.unwrap());
        let tags = Tags::parse(
            &imported_frame
                .tags
                .map(|tags| tags.join(","))
                .unwrap_or_default(),
        )
        .unwrap();
        let start = Utc.timestamp_opt(imported_frame.start_time.unwrap(), 0);
        let end = Utc.timestamp_opt(imported_frame.end_time.unwrap(), 0);
        Self::new(project, Some(tags), start.unwrap(), Some(end.unwrap())).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_project() {
        let entry = Entry::new(Project::new("project"), None, Utc::now(), None).unwrap();
        assert_eq!(entry.get_project(), Project::new("project"));
    }

    #[test]
    fn test_get_tags() {
        let tags = Tags::parse("tag1,tag2").unwrap();
        let entry = Entry::new("project".into(), Some(tags.clone()), Utc::now(), None).unwrap();
        assert_eq!(entry.get_tags().unwrap(), tags);
    }

    #[test]
    fn test_get_start() {
        let start = Utc::now();
        let entry = Entry::new("project".into(), None, start, None).unwrap();
        assert_eq!(entry.get_start(), start);
    }

    #[test]
    fn test_set_end() {
        let start = Utc::now();
        let mut entry = Entry::new("project".into(), None, start, None).unwrap();
        let end = Utc::now();
        entry.set_end(end);
        assert_eq!(entry.end.unwrap(), end);
    }

    #[test]
    fn test_is_running() {
        let start = Utc::now();
        let entry = Entry::new("project".into(), None, start, None).unwrap();
        assert!(entry.is_running());
    }

    #[test]
    fn test_get_id() {
        let entry = Entry::new("project".into(), None, Utc::now(), None).unwrap();
        assert_eq!(entry.get_id(), entry.id.to_string());
    }
}
