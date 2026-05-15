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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone, Utc};

    fn t(h: u32, m: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 15, h, m, 0).unwrap()
    }

    #[test]
    fn frame_new_stores_correct_fields() {
        let f = Frame::new("backend", vec!["api".into()], t(9, 0), t(10, 0));
        assert_eq!(f.project, "backend");
        assert_eq!(f.tags, vec!["api"]);
        assert_eq!(f.start, t(9, 0));
        assert_eq!(f.end, t(10, 0));
    }

    #[test]
    fn frame_new_assigns_unique_ids() {
        let f1 = Frame::new("p", vec![], t(9, 0), t(10, 0));
        let f2 = Frame::new("p", vec![], t(9, 0), t(10, 0));
        assert_ne!(f1.id, f2.id);
    }

    #[test]
    fn active_frame_stop_produces_correct_frame() {
        let active = ActiveFrame::new("backend", vec!["api".into()], t(9, 0));
        let frame = active.stop(t(10, 30));
        assert_eq!(frame.project, "backend");
        assert_eq!(frame.tags, vec!["api"]);
        assert_eq!(frame.start, t(9, 0));
        assert_eq!(frame.end, t(10, 30));
        assert_eq!(frame.end - frame.start, Duration::minutes(90));
    }
}
