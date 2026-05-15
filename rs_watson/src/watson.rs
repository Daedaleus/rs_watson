use chrono::{DateTime, Utc};
use rs_watson_storage::{ActiveFrameRecord, FrameRecord, Storage};
use thiserror::Error;
use uuid::Uuid;

use crate::{ActiveFrame, Frame};

#[derive(Debug, Error)]
pub enum WatsonError<E: std::error::Error + 'static> {
    #[error("Already tracking project \"{0}\"")]
    AlreadyTracking(String),
    #[error("Not currently tracking anything")]
    NotTracking,
    #[error("Frame not found")]
    FrameNotFound,
    #[error("Project \"{0}\" not found")]
    ProjectNotFound(String),
    #[error("End time must be after start time")]
    InvalidTimeRange,
    #[error("Time overlaps with existing frame for project \"{0}\"")]
    OverlappingFrame(String),
    #[error("Storage error: {0}")]
    Storage(E),
}

/// Returns the first record whose interval overlaps with [start, end).
/// When `end` is None, checks if `start` falls inside a frame.
/// `exclude` skips a specific frame by ID — used when editing an existing frame.
fn find_overlap(
    start: DateTime<Utc>,
    end: Option<DateTime<Utc>>,
    records: &[FrameRecord],
    exclude: Option<Uuid>,
) -> Option<&FrameRecord> {
    records.iter().find(|r| {
        if exclude.is_some_and(|id| r.id == id) {
            return false;
        }
        match end {
            Some(e) => start < r.end && r.start < e,
            None => start >= r.start && start < r.end,
        }
    })
}

/// Result of `Watson::start_or_replace`.
#[derive(Debug)]
pub struct StartResult {
    /// Frame that was automatically stopped to make room, if any.
    pub replaced: Option<Frame>,
    /// The newly started active frame.
    pub active: ActiveFrame,
}

pub struct Watson<S: Storage> {
    storage: S,
}

impl<S: Storage> Watson<S> {
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    // --- Private storage proxies -------------------------------------------
    // Wrapping every `self.storage.*` call avoids repeating `map_err(WatsonError::Storage)`.

    fn load_frames(&self) -> Result<Vec<FrameRecord>, WatsonError<S::Error>> {
        self.storage.load_frames().map_err(WatsonError::Storage)
    }

    fn save_frames(&self, frames: &[FrameRecord]) -> Result<(), WatsonError<S::Error>> {
        self.storage
            .save_frames(frames)
            .map_err(WatsonError::Storage)
    }

    fn load_active(&self) -> Result<Option<ActiveFrameRecord>, WatsonError<S::Error>> {
        self.storage.load_active().map_err(WatsonError::Storage)
    }

    fn save_active(&self, frame: Option<&ActiveFrameRecord>) -> Result<(), WatsonError<S::Error>> {
        self.storage
            .save_active(frame)
            .map_err(WatsonError::Storage)
    }

    /// Loads frames, applies a mutation via `f`, and saves the result.
    /// Frames are only written if `f` succeeds.
    fn modify_frames<T, F>(&self, f: F) -> Result<T, WatsonError<S::Error>>
    where
        F: FnOnce(&mut Vec<FrameRecord>) -> Result<T, WatsonError<S::Error>>,
    {
        let mut records = self.load_frames()?;
        let result = f(&mut records)?;
        self.save_frames(&records)?;
        Ok(result)
    }

    /// Validates `at` against `records`, then saves a new active frame.
    /// Shared by `start` and `start_or_replace`.
    fn begin_tracking(
        &self,
        project: impl Into<String>,
        tags: Vec<String>,
        at: DateTime<Utc>,
        records: &[FrameRecord],
    ) -> Result<ActiveFrame, WatsonError<S::Error>> {
        if let Some(conflict) = find_overlap(at, None, records, None) {
            return Err(WatsonError::OverlappingFrame(conflict.project.clone()));
        }
        let active = ActiveFrame::new(project, tags, at);
        self.save_active(Some(&ActiveFrameRecord::from(&active)))?;
        Ok(active)
    }

    // --- Public API --------------------------------------------------------

    pub fn start(
        &self,
        project: impl Into<String>,
        tags: Vec<String>,
        at: DateTime<Utc>,
    ) -> Result<ActiveFrame, WatsonError<S::Error>> {
        if let Some(active) = self.load_active()? {
            return Err(WatsonError::AlreadyTracking(active.project));
        }
        let records = self.load_frames()?;
        self.begin_tracking(project, tags, at, &records)
    }

    /// Starts tracking. If another frame is already active it is automatically
    /// stopped at `at` before the new frame begins.
    pub fn start_or_replace(
        &self,
        project: impl Into<String>,
        tags: Vec<String>,
        at: DateTime<Utc>,
    ) -> Result<StartResult, WatsonError<S::Error>> {
        let existing_active = self.load_active()?;
        let mut records = self.load_frames()?;

        let replaced = if let Some(active_record) = existing_active {
            let active = ActiveFrame::from(active_record);
            if at <= active.start {
                return Err(WatsonError::InvalidTimeRange);
            }
            let completed = active.stop(at);
            if let Some(conflict) =
                find_overlap(completed.start, Some(completed.end), &records, None)
            {
                return Err(WatsonError::OverlappingFrame(conflict.project.clone()));
            }
            records.push(FrameRecord::from(&completed));
            Some(completed)
        } else {
            None
        };

        if replaced.is_some() {
            self.save_frames(&records)?;
        }

        let active = self.begin_tracking(project, tags, at, &records)?;
        Ok(StartResult { replaced, active })
    }

    pub fn stop(&self, at: DateTime<Utc>) -> Result<Frame, WatsonError<S::Error>> {
        let active = self.load_active()?.ok_or(WatsonError::NotTracking)?;
        if at <= active.start {
            return Err(WatsonError::InvalidTimeRange);
        }
        let frame = ActiveFrame::from(active).stop(at);
        self.modify_frames(|records| {
            if let Some(conflict) = find_overlap(frame.start, Some(frame.end), records, None) {
                return Err(WatsonError::OverlappingFrame(conflict.project.clone()));
            }
            records.push(FrameRecord::from(&frame));
            Ok(())
        })?;
        self.save_active(None)?;
        Ok(frame)
    }

    pub fn cancel(&self) -> Result<ActiveFrame, WatsonError<S::Error>> {
        let active = self.load_active()?.ok_or(WatsonError::NotTracking)?;
        self.save_active(None)?;
        Ok(ActiveFrame::from(active))
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
        self.modify_frames(|records| {
            if let Some(conflict) = find_overlap(frame.start, Some(frame.end), records, None) {
                return Err(WatsonError::OverlappingFrame(conflict.project.clone()));
            }
            records.push(FrameRecord::from(&frame));
            Ok(())
        })?;
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
        if end <= start {
            return Err(WatsonError::InvalidTimeRange);
        }
        let frame = Frame {
            id,
            project: project.into(),
            tags,
            start,
            end,
        };
        self.modify_frames(|records| {
            let pos = records
                .iter()
                .position(|r| r.id == id)
                .ok_or(WatsonError::FrameNotFound)?;
            if let Some(conflict) = find_overlap(frame.start, Some(frame.end), records, Some(id)) {
                return Err(WatsonError::OverlappingFrame(conflict.project.clone()));
            }
            records[pos] = FrameRecord::from(&frame);
            Ok(())
        })?;
        Ok(frame)
    }

    pub fn remove(&self, id: Uuid) -> Result<Frame, WatsonError<S::Error>> {
        self.modify_frames(|records| {
            let pos = records
                .iter()
                .position(|r| r.id == id)
                .ok_or(WatsonError::FrameNotFound)?;
            Ok(Frame::from(records.remove(pos)))
        })
    }

    pub fn rename(
        &self,
        from: &str,
        to: impl Into<String>,
    ) -> Result<usize, WatsonError<S::Error>> {
        let to = to.into();

        let active_updated = if let Some(mut active) = self.load_active()? {
            if active.project == from {
                active.project = to.clone();
                self.save_active(Some(&active))?;
                true
            } else {
                false
            }
        } else {
            false
        };

        let frame_count = self.modify_frames(|records| {
            let mut count = 0usize;
            for record in records.iter_mut() {
                if record.project == from {
                    record.project = to.clone();
                    count += 1;
                }
            }
            if count == 0 && !active_updated {
                return Err(WatsonError::ProjectNotFound(from.to_string()));
            }
            Ok(count)
        })?;

        Ok(frame_count + usize::from(active_updated))
    }

    /// Imports a list of frames, appending them sorted by start time.
    /// Does not check for overlaps — suitable for bulk migration.
    pub fn import_frames(&self, frames: Vec<Frame>) -> Result<usize, WatsonError<S::Error>> {
        let count = frames.len();
        self.modify_frames(|records| {
            records.extend(frames.iter().map(FrameRecord::from));
            records.sort_by_key(|r| r.start);
            Ok(count)
        })
    }

    pub fn status(&self) -> Result<Option<ActiveFrame>, WatsonError<S::Error>> {
        Ok(self.load_active()?.map(ActiveFrame::from))
    }

    pub fn log(&self) -> Result<Vec<Frame>, WatsonError<S::Error>> {
        let mut frames: Vec<Frame> = self.load_frames()?.into_iter().map(Frame::from).collect();
        frames.sort_by_key(|f| f.start);
        Ok(frames)
    }

    pub fn projects(&self) -> Result<Vec<String>, WatsonError<S::Error>> {
        let mut names: Vec<String> = self.load_frames()?.into_iter().map(|r| r.project).collect();
        names.sort();
        names.dedup();
        Ok(names)
    }

    pub fn tags(&self) -> Result<Vec<String>, WatsonError<S::Error>> {
        let mut tags: Vec<String> = self
            .load_frames()?
            .into_iter()
            .flat_map(|r| r.tags)
            .collect();
        tags.sort();
        tags.dedup();
        Ok(tags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::MemoryStorage;
    use chrono::{Duration, TimeZone, Utc};

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
        assert!(matches!(
            w.stop(t(10, 0)).unwrap_err(),
            WatsonError::NotTracking
        ));
    }

    #[test]
    fn stop_rejects_at_before_start() {
        let w = w();
        w.start("backend", vec![], t(10, 0)).unwrap();
        assert!(matches!(
            w.stop(t(9, 0)).unwrap_err(),
            WatsonError::InvalidTimeRange
        ));
    }

    #[test]
    fn stop_rejects_at_equal_to_start() {
        let w = w();
        w.start("backend", vec![], t(10, 0)).unwrap();
        assert!(matches!(
            w.stop(t(10, 0)).unwrap_err(),
            WatsonError::InvalidTimeRange
        ));
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
        let f = w
            .add("backend", vec!["api".into()], t(9, 0), t(10, 0))
            .unwrap();
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
            .edit(
                original.id,
                "frontend",
                vec!["ui".into()],
                t(9, 30),
                t(11, 0),
            )
            .unwrap();
        assert_eq!(updated.project, "frontend");
        assert_eq!(updated.tags, vec!["ui"]);
        assert_eq!(w.log().unwrap()[0].project, "frontend");
    }

    #[test]
    fn edit_unknown_id_returns_error() {
        let w = w();
        assert!(matches!(
            w.edit(Uuid::new_v4(), "x", vec![], t(9, 0), t(10, 0))
                .unwrap_err(),
            WatsonError::FrameNotFound
        ));
    }

    #[test]
    fn edit_rejects_end_before_start() {
        let w = w();
        let frame = w.add("backend", vec![], t(9, 0), t(10, 0)).unwrap();
        assert!(matches!(
            w.edit(frame.id, "backend", vec![], t(10, 0), t(9, 0))
                .unwrap_err(),
            WatsonError::InvalidTimeRange
        ));
    }

    #[test]
    fn edit_rejects_equal_start_and_end() {
        let w = w();
        let frame = w.add("backend", vec![], t(9, 0), t(10, 0)).unwrap();
        assert!(matches!(
            w.edit(frame.id, "backend", vec![], t(9, 0), t(9, 0))
                .unwrap_err(),
            WatsonError::InvalidTimeRange
        ));
    }

    // --- remove ---

    #[test]
    fn remove_deletes_frame() {
        let w = w();
        let frame = w.add("backend", vec![], t(9, 0), t(10, 0)).unwrap();
        w.remove(frame.id).unwrap();
        assert!(w.log().unwrap().is_empty());
    }

    #[test]
    fn remove_returns_deleted_frame() {
        let w = w();
        let frame = w
            .add("backend", vec!["api".into()], t(9, 0), t(10, 0))
            .unwrap();
        let removed = w.remove(frame.id).unwrap();
        assert_eq!(removed.project, "backend");
        assert_eq!(removed.tags, vec!["api"]);
    }

    #[test]
    fn remove_unknown_id_returns_error() {
        let w = w();
        assert!(matches!(
            w.remove(Uuid::new_v4()).unwrap_err(),
            WatsonError::FrameNotFound
        ));
    }

    #[test]
    fn remove_does_not_affect_active_frame() {
        let w = w();
        w.start("active", vec![], t(9, 0)).unwrap();
        assert!(matches!(
            w.remove(Uuid::new_v4()).unwrap_err(),
            WatsonError::FrameNotFound
        ));
        assert!(w.status().unwrap().is_some());
    }

    // --- rename ---

    #[test]
    fn rename_updates_all_matching_frames() {
        let w = w();
        w.add("old", vec![], t(9, 0), t(10, 0)).unwrap();
        w.add("old", vec![], t(10, 0), t(11, 0)).unwrap();
        w.add("other", vec![], t(11, 0), t(12, 0)).unwrap();
        let count = w.rename("old", "new").unwrap();
        assert_eq!(count, 2);
        let names: Vec<_> = w.log().unwrap().into_iter().map(|f| f.project).collect();
        assert_eq!(names, vec!["new", "new", "other"]);
    }

    #[test]
    fn rename_updates_active_frame_if_tracked() {
        let w = w();
        w.start("old", vec![], t(9, 0)).unwrap();
        let count = w.rename("old", "new").unwrap();
        assert_eq!(count, 1);
        assert_eq!(w.status().unwrap().unwrap().project, "new");
    }

    #[test]
    fn rename_unknown_project_returns_error() {
        let w = w();
        assert!(matches!(
            w.rename("ghost", "new").unwrap_err(),
            WatsonError::ProjectNotFound(_)
        ));
    }

    // --- import_frames ---

    #[test]
    fn import_frames_appends_and_sorts_by_start() {
        let w = w();
        let f1 = Frame::new("a", vec![], t(10, 0), t(11, 0));
        let f2 = Frame::new("b", vec![], t(8, 0), t(9, 0));
        let count = w.import_frames(vec![f1, f2]).unwrap();
        assert_eq!(count, 2);
        let log = w.log().unwrap();
        assert_eq!(log[0].project, "b");
        assert_eq!(log[1].project, "a");
    }

    #[test]
    fn import_frames_appends_to_existing() {
        let w = w();
        w.add("existing", vec![], t(7, 0), t(8, 0)).unwrap();
        w.import_frames(vec![Frame::new("imported", vec![], t(9, 0), t(10, 0))])
            .unwrap();
        assert_eq!(w.log().unwrap().len(), 2);
    }

    // --- log ---

    #[test]
    fn log_returns_frames_sorted_by_start_ascending() {
        let w = w();
        w.add("second", vec![], t(10, 0), t(11, 0)).unwrap();
        w.add("first", vec![], t(8, 0), t(9, 0)).unwrap();
        let frames = w.log().unwrap();
        assert_eq!(frames[0].project, "first");
        assert_eq!(frames[1].project, "second");
    }

    // --- projects ---

    #[test]
    fn projects_returns_unique_sorted_names() {
        let w = w();
        w.add("backend", vec![], t(9, 0), t(10, 0)).unwrap();
        w.add("frontend", vec![], t(10, 0), t(11, 0)).unwrap();
        w.add("backend", vec![], t(11, 0), t(12, 0)).unwrap();
        assert_eq!(w.projects().unwrap(), vec!["backend", "frontend"]);
    }

    // --- tags ---

    #[test]
    fn tags_returns_unique_sorted_tags() {
        let w = w();
        w.add("a", vec!["beta".into(), "alpha".into()], t(9, 0), t(10, 0))
            .unwrap();
        w.add(
            "b",
            vec!["alpha".into(), "gamma".into()],
            t(10, 0),
            t(11, 0),
        )
        .unwrap();
        assert_eq!(w.tags().unwrap(), vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn tags_returns_empty_when_no_frames() {
        assert!(w().tags().unwrap().is_empty());
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

    // --- start_or_replace ---

    #[test]
    fn start_or_replace_with_no_active_behaves_like_start() {
        let w = w();
        let result = w.start_or_replace("backend", vec![], t(9, 0)).unwrap();
        assert!(result.replaced.is_none());
        assert_eq!(result.active.project, "backend");
        assert!(w.status().unwrap().is_some());
    }

    #[test]
    fn start_or_replace_stops_active_and_starts_new() {
        let w = w();
        w.start("old", vec![], t(9, 0)).unwrap();
        let result = w.start_or_replace("new", vec![], t(10, 0)).unwrap();

        let stopped = result.replaced.unwrap();
        assert_eq!(stopped.project, "old");
        assert_eq!(stopped.end, t(10, 0));
        assert_eq!(result.active.project, "new");
        assert_eq!(w.log().unwrap().len(), 1);
        assert_eq!(w.status().unwrap().unwrap().project, "new");
    }

    #[test]
    fn start_or_replace_rejects_at_before_active_start() {
        let w = w();
        w.start("old", vec![], t(10, 0)).unwrap();
        assert!(matches!(
            w.start_or_replace("new", vec![], t(9, 0)).unwrap_err(),
            WatsonError::InvalidTimeRange
        ));
    }

    #[test]
    fn start_or_replace_rejects_overlap_with_existing_frame() {
        let w = w();
        w.add("existing", vec![], t(10, 0), t(11, 0)).unwrap();
        w.start("active", vec![], t(9, 0)).unwrap();
        assert!(matches!(
            w.start_or_replace("new", vec![], t(10, 30)).unwrap_err(),
            WatsonError::OverlappingFrame(_)
        ));
    }

    #[test]
    fn start_or_replace_with_at_equal_to_existing_end_is_adjacent_and_ok() {
        let w = w();
        w.add("first", vec![], t(8, 0), t(9, 0)).unwrap();
        w.start("second", vec![], t(9, 0)).unwrap();
        let result = w.start_or_replace("third", vec![], t(10, 0)).unwrap();
        assert!(result.replaced.is_some());
        assert_eq!(result.active.project, "third");
    }

    // --- overlap ---

    #[test]
    fn add_rejects_overlap_with_existing_frame() {
        let w = w();
        w.add("backend", vec![], t(9, 0), t(11, 0)).unwrap();
        let err = w.add("frontend", vec![], t(10, 0), t(12, 0)).unwrap_err();
        assert!(matches!(err, WatsonError::OverlappingFrame(_)));
    }

    #[test]
    fn start_rejects_time_inside_existing_frame() {
        let w = w();
        w.add("backend", vec![], t(9, 0), t(11, 0)).unwrap();
        let err = w.start("frontend", vec![], t(10, 0)).unwrap_err();
        assert!(matches!(err, WatsonError::OverlappingFrame(_)));
    }

    #[test]
    fn add_adjacent_frames_do_not_overlap() {
        let w = w();
        w.add("backend", vec![], t(9, 0), t(10, 0)).unwrap();
        assert!(w.add("frontend", vec![], t(10, 0), t(11, 0)).is_ok());
    }

    #[test]
    fn edit_rejects_new_times_that_overlap_another_frame() {
        let w = w();
        let f1 = w.add("backend", vec![], t(9, 0), t(10, 0)).unwrap();
        w.add("frontend", vec![], t(11, 0), t(12, 0)).unwrap();
        let err = w
            .edit(f1.id, "backend", vec![], t(9, 0), t(11, 30))
            .unwrap_err();
        assert!(matches!(err, WatsonError::OverlappingFrame(_)));
    }

    #[test]
    fn edit_allows_keeping_same_times() {
        let w = w();
        let frame = w.add("backend", vec![], t(9, 0), t(10, 0)).unwrap();
        assert!(
            w.edit(frame.id, "backend-renamed", vec![], t(9, 0), t(10, 0))
                .is_ok()
        );
    }

    #[test]
    fn stop_rejects_time_that_would_overlap_existing_frame() {
        let w = w();
        w.add("other", vec![], t(10, 0), t(11, 0)).unwrap();
        w.start("backend", vec![], t(9, 0)).unwrap();
        let err = w.stop(t(10, 30)).unwrap_err();
        assert!(matches!(err, WatsonError::OverlappingFrame(_)));
    }
}
