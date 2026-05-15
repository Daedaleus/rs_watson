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

    pub fn remove(&self, id: Uuid) -> Result<Frame, WatsonError<S::Error>> {
        let mut records = self.storage.load_frames().map_err(WatsonError::Storage)?;
        let pos = records
            .iter()
            .position(|r| r.id == id)
            .ok_or(WatsonError::FrameNotFound)?;
        let removed = Frame::from(records.remove(pos));
        self.storage.save_frames(&records).map_err(WatsonError::Storage)?;
        Ok(removed)
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


#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use chrono::{Duration, TimeZone, Utc};
    use rs_watson_storage::{ActiveFrameRecord, FrameRecord, Storage};

    struct MemoryStorage {
        frames: RefCell<Vec<FrameRecord>>,
        active: RefCell<Option<ActiveFrameRecord>>,
    }

    impl MemoryStorage {
        fn new() -> Self {
            Self {
                frames: RefCell::new(Vec::new()),
                active: RefCell::new(None),
            }
        }
    }

    impl Storage for MemoryStorage {
        type Error = std::convert::Infallible;

        fn load_frames(&self) -> Result<Vec<FrameRecord>, Self::Error> {
            Ok(self.frames.borrow().clone())
        }
        fn save_frames(&self, frames: &[FrameRecord]) -> Result<(), Self::Error> {
            *self.frames.borrow_mut() = frames.to_vec();
            Ok(())
        }
        fn load_active(&self) -> Result<Option<ActiveFrameRecord>, Self::Error> {
            Ok(self.active.borrow().clone())
        }
        fn save_active(&self, frame: Option<&ActiveFrameRecord>) -> Result<(), Self::Error> {
            *self.active.borrow_mut() = frame.cloned();
            Ok(())
        }
    }

    fn w() -> Watson<MemoryStorage> {
        Watson::new(MemoryStorage::new())
    }

    fn t(h: u32, m: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 15, h, m, 0).unwrap()
    }

    // --- start ---

    #[test]
    fn start_creates_active_frame() {
        let w = w();
        let f = w.start("backend", vec!["api".into()], t(9, 0)).unwrap();
        assert_eq!(f.project, "backend");
        assert_eq!(f.tags, vec!["api"]);
        assert_eq!(f.start, t(9, 0));
    }

    #[test]
    fn start_when_already_tracking_returns_error() {
        let w = w();
        w.start("backend", vec![], t(9, 0)).unwrap();
        let err = w.start("frontend", vec![], t(10, 0)).unwrap_err();
        assert!(matches!(err, WatsonError::AlreadyTracking(_)));
    }

    // --- stop ---

    #[test]
    fn stop_produces_correct_frame() {
        let w = w();
        w.start("backend", vec![], t(9, 0)).unwrap();
        let f = w.stop(t(10, 30)).unwrap();
        assert_eq!(f.project, "backend");
        assert_eq!(f.start, t(9, 0));
        assert_eq!(f.end, t(10, 30));
        assert_eq!(f.end - f.start, Duration::minutes(90));
    }

    #[test]
    fn stop_clears_active_and_saves_frame() {
        let w = w();
        w.start("backend", vec![], t(9, 0)).unwrap();
        w.stop(t(10, 0)).unwrap();
        assert!(w.status().unwrap().is_none());
        assert_eq!(w.log().unwrap().len(), 1);
    }

    #[test]
    fn stop_when_not_tracking_returns_error() {
        let w = w();
        assert!(matches!(w.stop(t(10, 0)).unwrap_err(), WatsonError::NotTracking));
    }

    // --- cancel ---

    #[test]
    fn cancel_clears_active_without_saving_frame() {
        let w = w();
        w.start("backend", vec![], t(9, 0)).unwrap();
        let f = w.cancel().unwrap();
        assert_eq!(f.project, "backend");
        assert!(w.status().unwrap().is_none());
        assert!(w.log().unwrap().is_empty());
    }

    #[test]
    fn cancel_when_not_tracking_returns_error() {
        let w = w();
        assert!(matches!(w.cancel().unwrap_err(), WatsonError::NotTracking));
    }

    // --- add ---

    #[test]
    fn add_creates_saved_frame() {
        let w = w();
        let f = w.add("backend", vec!["api".into()], t(9, 0), t(10, 0)).unwrap();
        assert_eq!(f.project, "backend");
        assert_eq!(f.end - f.start, Duration::hours(1));
        assert_eq!(w.log().unwrap().len(), 1);
    }

    #[test]
    fn add_rejects_end_before_start() {
        let w = w();
        assert!(matches!(
            w.add("backend", vec![], t(10, 0), t(9, 0)).unwrap_err(),
            WatsonError::InvalidTimeRange
        ));
    }

    #[test]
    fn add_rejects_equal_start_and_end() {
        let w = w();
        assert!(matches!(
            w.add("backend", vec![], t(9, 0), t(9, 0)).unwrap_err(),
            WatsonError::InvalidTimeRange
        ));
    }

    // --- edit ---

    #[test]
    fn edit_updates_stored_frame() {
        let w = w();
        let original = w.add("backend", vec![], t(9, 0), t(10, 0)).unwrap();
        let updated = w
            .edit(original.id, "frontend", vec!["ui".into()], t(9, 30), t(11, 0))
            .unwrap();
        assert_eq!(updated.project, "frontend");
        assert_eq!(updated.tags, vec!["ui"]);
        assert_eq!(w.log().unwrap()[0].project, "frontend");
    }

    #[test]
    fn edit_unknown_id_returns_error() {
        let w = w();
        assert!(matches!(
            w.edit(Uuid::new_v4(), "x", vec![], t(9, 0), t(10, 0)).unwrap_err(),
            WatsonError::FrameNotFound
        ));
    }

    // --- log ---

    #[test]
    fn log_returns_frames_sorted_by_start_ascending() {
        let w = w();
        w.add("second", vec![], t(10, 0), t(11, 0)).unwrap();
        w.add("first",  vec![], t(8,  0), t(9,  0)).unwrap();
        let frames = w.log().unwrap();
        assert_eq!(frames[0].project, "first");
        assert_eq!(frames[1].project, "second");
    }

    // --- projects ---

    #[test]
    fn projects_returns_unique_sorted_names() {
        let w = w();
        w.add("backend",  vec![], t(9,  0), t(10, 0)).unwrap();
        w.add("frontend", vec![], t(10, 0), t(11, 0)).unwrap();
        w.add("backend",  vec![], t(11, 0), t(12, 0)).unwrap();
        assert_eq!(w.projects().unwrap(), vec!["backend", "frontend"]);
    }

    // --- status ---

    #[test]
    fn status_returns_none_when_idle() {
        assert!(w().status().unwrap().is_none());
    }

    #[test]
    fn status_returns_current_project() {
        let w = w();
        w.start("backend", vec![], t(9, 0)).unwrap();
        let active = w.status().unwrap().unwrap();
        assert_eq!(active.project, "backend");
        assert_eq!(active.start, t(9, 0));
    }
}
