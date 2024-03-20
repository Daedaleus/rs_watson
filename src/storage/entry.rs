use std::fmt::Display;

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

    pub fn print_report(&self) -> String {
        let tags = self.tags.clone().unwrap_or_default().join(", ");
        let start = self.start.format("%H:%M");
        let end = match self.end {
            Some(end) => end.format("%H:%M").to_string(),
            None => "now".to_string(),
        };
        format!("[{}   {} - {}]", tags, start, end)
    }

    pub fn get_project(&self) -> String {
        self.project.clone()
    }

    pub fn get_tags(&self) -> Option<Vec<String>> {
        self.tags.clone()
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
            // f35bb24  09:26 to 10:22  apollo11    [reactor, brakes, steering, wheels, module]
            "{}  {} to {}  {}    [{}]",
            id,
            self.start.format("%H:%M"),
            end,
            self.project,
            tags
        )
    }
}
