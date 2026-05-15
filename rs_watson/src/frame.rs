use chrono::{DateTime, Utc};
use uuid::Uuid;

/// A completed time tracking entry with a definite start and end.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub id: Uuid,
    pub project: String,
    pub tags: Vec<String>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl Frame {
    pub fn new(
        project: impl Into<String>,
        tags: Vec<String>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            project: project.into(),
            tags,
            start,
            end,
        }
    }
}

/// A currently running time tracking entry — no end time yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveFrame {
    pub project: String,
    pub tags: Vec<String>,
    pub start: DateTime<Utc>,
}

impl ActiveFrame {
    pub fn new(project: impl Into<String>, tags: Vec<String>, start: DateTime<Utc>) -> Self {
        Self {
            project: project.into(),
            tags,
            start,
        }
    }

    /// Stops the active frame at `end`, producing a completed [`Frame`].
    pub fn stop(self, end: DateTime<Utc>) -> Frame {
        Frame::new(self.project, self.tags, self.start, end)
    }
}
