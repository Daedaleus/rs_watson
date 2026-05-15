use chrono::{DateTime, Utc};
use rs_watson_storage::{ActiveFrameRecord, FrameRecord, Storage};
use uuid::Uuid;
use thiserror::Error;

use crate::{ActiveFrame, Frame};

#[derive(Debug, Error)]
pub enum WatsonError<E: std::error::Error + 'static> {
    #[error("Already tracking project \"{0}\"")]
    AlreadyTracking(String),
    #[error("Not currently tracking anything")]
    NotTracking,
    #[error("Frame not found")]
    FrameNotFound,
    #[error("End time must be after start time")]
    InvalidTimeRange,
    #[error("Storage error: {0}")]
    Storage(E),
}

pub struct Watson<S: Storage> {
    storage: S,
}

impl<S: Storage> Watson<S> {
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    pub fn add(
        &self,
        project: impl Into<String>,
        tags: Vec<String>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Frame, WatsonError<S::Error>> {
        if end <= start {
            return Err(WatsonError::InvalidTimeRange);
        }
        let frame = Frame::new(project, tags, start, end);
        let mut records = self.storage.load_frames().map_err(WatsonError::Storage)?;
        records.push(FrameRecord::from(&frame));
        self.storage.save_frames(&records).map_err(WatsonError::Storage)?;
        Ok(frame)
    }

    pub fn edit(
        &self,
        id: Uuid,
        project: impl Into<String>,
        tags: Vec<String>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Frame, WatsonError<S::Error>> {
        let mut records = self.storage.load_frames().map_err(WatsonError::Storage)?;
        let pos = records
            .iter()
            .position(|r| r.id == id)
            .ok_or(WatsonError::FrameNotFound)?;
        let frame = Frame { id, project: project.into(), tags, start, end };
        records[pos] = FrameRecord::from(&frame);
        self.storage.save_frames(&records).map_err(WatsonError::Storage)?;
        Ok(frame)
    }

    pub fn cancel(&self) -> Result<ActiveFrame, WatsonError<S::Error>> {
        let active = self
            .storage
            .load_active()
            .map_err(WatsonError::Storage)?
            .ok_or(WatsonError::NotTracking)?;
        self.storage.save_active(None).map_err(WatsonError::Storage)?;
        Ok(ActiveFrame::from(active))
    }

    pub fn projects(&self) -> Result<Vec<String>, WatsonError<S::Error>> {
        let mut names: Vec<String> = self
            .storage
            .load_frames()
            .map_err(WatsonError::Storage)?
            .into_iter()
            .map(|r| r.project)
            .collect();
        names.sort();
        names.dedup();
        Ok(names)
    }

    pub fn log(&self) -> Result<Vec<Frame>, WatsonError<S::Error>> {
        let mut frames: Vec<Frame> = self
            .storage
            .load_frames()
            .map_err(WatsonError::Storage)?
            .into_iter()
            .map(Frame::from)
            .collect();
        frames.sort_by_key(|f| f.start);
        Ok(frames)
    }

    pub fn status(&self) -> Result<Option<ActiveFrame>, WatsonError<S::Error>> {
        Ok(self
            .storage
            .load_active()
            .map_err(WatsonError::Storage)?
            .map(ActiveFrame::from))
    }

    pub fn stop(&self, at: DateTime<Utc>) -> Result<Frame, WatsonError<S::Error>> {
        let active = self
            .storage
            .load_active()
            .map_err(WatsonError::Storage)?
            .ok_or(WatsonError::NotTracking)?;

        let frame = ActiveFrame::from(active).stop(at);

        let mut frames = self.storage.load_frames().map_err(WatsonError::Storage)?;
        frames.push(FrameRecord::from(&frame));
        self.storage.save_frames(&frames).map_err(WatsonError::Storage)?;
        self.storage.save_active(None).map_err(WatsonError::Storage)?;

        Ok(frame)
    }

    pub fn start(
        &self,
        project: impl Into<String>,
        tags: Vec<String>,
        at: DateTime<Utc>,
    ) -> Result<ActiveFrame, WatsonError<S::Error>> {
        if let Some(active) = self.storage.load_active().map_err(WatsonError::Storage)? {
            return Err(WatsonError::AlreadyTracking(active.project));
        }
        let frame = ActiveFrame::new(project, tags, at);
        let record = ActiveFrameRecord::from(&frame);
        self.storage.save_active(Some(&record)).map_err(WatsonError::Storage)?;
        Ok(frame)
    }
}

// --- Conversions ---

impl From<&ActiveFrame> for ActiveFrameRecord {
    fn from(f: &ActiveFrame) -> Self {
        ActiveFrameRecord {
            project: f.project.clone(),
            tags: f.tags.clone(),
            start: f.start,
        }
    }
}

impl From<ActiveFrameRecord> for ActiveFrame {
    fn from(r: ActiveFrameRecord) -> Self {
        ActiveFrame {
            project: r.project,
            tags: r.tags,
            start: r.start,
        }
    }
}

impl From<&Frame> for FrameRecord {
    fn from(f: &Frame) -> Self {
        FrameRecord {
            id: f.id,
            project: f.project.clone(),
            tags: f.tags.clone(),
            start: f.start,
            end: f.end,
        }
    }
}

impl From<FrameRecord> for Frame {
    fn from(r: FrameRecord) -> Self {
        Frame {
            id: r.id,
            project: r.project,
            tags: r.tags,
            start: r.start,
            end: r.end,
        }
    }
}
