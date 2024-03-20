use std::fmt::Display;

use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::storage::gen_id;

#[derive(Serialize, Deserialize, Clone)]
pub struct Entry {
    id: String,
    project: String,
    tags: Option<Vec<String>>,
    start: DateTime<Utc>,
    end: Option<DateTime<Utc>>,
}

impl Entry {
    pub fn new(
        project: String,
        tags: Option<Vec<String>>,
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
        let tags = self.tags.clone().unwrap_or_default().join(", ");
        let duration = self.get_duration_as_human_readable()?;
        Ok(format!("[{}   {}]", tags, duration))
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

    pub fn get_project(&self) -> String {
        self.project.clone()
    }

    pub fn get_tags(&self) -> Option<Vec<String>> {
        self.tags.clone()
    }

    pub fn get_tags_as_string(&self) -> String {
        self.tags.clone().unwrap_or_default().join(", ")
    }

    pub fn get_start(&self) -> DateTime<Utc> {
        self.start
    }

    pub fn set_end(&mut self, end: DateTime<Utc>) {
        self.end = Some(end);
    }
}

impl Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tags = self.tags.clone().unwrap_or_default().join(", ");
        let id = &self.id[..8];
        let end = match self.end {
            Some(end) => end.format("%H:%M").to_string(),
            None => "now".to_string(),
        };
        write!(
            f,
            "{}  {} to {}  {}    [{}]",
            id,
            self.start.format("%H:%M"),
            end,
            self.project,
            tags
        )
    }
}
