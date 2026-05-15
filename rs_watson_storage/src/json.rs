use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::{ActiveFrameRecord, FrameRecord, Storage};

#[derive(Debug, Error)]
pub enum JsonStorageError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub struct JsonStorage {
    frames_path: PathBuf,
    state_path: PathBuf,
}

impl JsonStorage {
    pub fn new(dir: impl AsRef<Path>) -> Self {
        let dir = dir.as_ref();
        Self {
            frames_path: dir.join("frames.json"),
            state_path: dir.join("state.json"),
        }
    }
}

/// Writes `data` to a `.tmp` sibling of `path`, then renames it into place.
/// `rename` is atomic on POSIX when src and dst are on the same filesystem,
/// so a crash mid-write never leaves a partial file at the real path.
fn write_atomic(path: &Path, data: &str) -> std::io::Result<()> {
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, data)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

impl Storage for JsonStorage {
    type Error = JsonStorageError;

    fn load_frames(&self) -> Result<Vec<FrameRecord>, Self::Error> {
        if !self.frames_path.exists() {
            return Ok(Vec::new());
        }
        let data = fs::read_to_string(&self.frames_path)?;
        Ok(serde_json::from_str(&data)?)
    }

    fn save_frames(&self, frames: &[FrameRecord]) -> Result<(), Self::Error> {
        let data = serde_json::to_string_pretty(frames)?;
        write_atomic(&self.frames_path, &data)?;
        Ok(())
    }

    fn load_active(&self) -> Result<Option<ActiveFrameRecord>, Self::Error> {
        if !self.state_path.exists() {
            return Ok(None);
        }
        let data = fs::read_to_string(&self.state_path)?;
        Ok(serde_json::from_str(&data)?)
    }

    fn save_active(&self, frame: Option<&ActiveFrameRecord>) -> Result<(), Self::Error> {
        match frame {
            Some(f) => {
                let data = serde_json::to_string_pretty(f)?;
                write_atomic(&self.state_path, &data)?;
            }
            None => {
                if self.state_path.exists() {
                    fs::remove_file(&self.state_path)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    fn t(h: u32) -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 15, h, 0, 0).unwrap()
    }

    fn sample_frame() -> FrameRecord {
        FrameRecord {
            id: Uuid::new_v4(),
            project: "backend".into(),
            tags: vec!["api".into()],
            start: t(9),
            end: t(10),
        }
    }

    fn sample_active() -> ActiveFrameRecord {
        ActiveFrameRecord {
            project: "frontend".into(),
            tags: vec!["ui".into()],
            start: t(9),
        }
    }

    #[test]
    fn frames_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let s = JsonStorage::new(dir.path());
        let frames = vec![sample_frame()];
        s.save_frames(&frames).unwrap();
        let loaded = s.load_frames().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].project, "backend");
        assert_eq!(loaded[0].tags, vec!["api"]);
        assert_eq!(loaded[0].start, t(9));
        assert_eq!(loaded[0].end,   t(10));
    }

    #[test]
    fn frames_id_preserved_in_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let s = JsonStorage::new(dir.path());
        let frame = sample_frame();
        let id = frame.id;
        s.save_frames(&[frame]).unwrap();
        assert_eq!(s.load_frames().unwrap()[0].id, id);
    }

    #[test]
    fn load_frames_missing_file_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        assert!(JsonStorage::new(dir.path()).load_frames().unwrap().is_empty());
    }

    #[test]
    fn active_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let s = JsonStorage::new(dir.path());
        s.save_active(Some(&sample_active())).unwrap();
        let loaded = s.load_active().unwrap().unwrap();
        assert_eq!(loaded.project, "frontend");
        assert_eq!(loaded.tags, vec!["ui"]);
        assert_eq!(loaded.start, t(9));
    }

    #[test]
    fn load_active_missing_file_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        assert!(JsonStorage::new(dir.path()).load_active().unwrap().is_none());
    }

    #[test]
    fn save_active_none_clears_state() {
        let dir = tempfile::tempdir().unwrap();
        let s = JsonStorage::new(dir.path());
        s.save_active(Some(&sample_active())).unwrap();
        s.save_active(None).unwrap();
        assert!(s.load_active().unwrap().is_none());
        assert!(!dir.path().join("state.json").exists());
    }

    #[test]
    fn save_frames_multiple_and_load_all() {
        let dir = tempfile::tempdir().unwrap();
        let s = JsonStorage::new(dir.path());
        let frames = vec![sample_frame(), sample_frame()];
        s.save_frames(&frames).unwrap();
        assert_eq!(s.load_frames().unwrap().len(), 2);
    }

    #[test]
    fn no_tmp_file_left_after_successful_write() {
        let dir = tempfile::tempdir().unwrap();
        let s = JsonStorage::new(dir.path());
        s.save_frames(&[sample_frame()]).unwrap();
        assert!(!dir.path().join("frames.json.tmp").exists());
    }
}
