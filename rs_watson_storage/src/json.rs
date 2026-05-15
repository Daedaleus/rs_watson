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
        fs::write(&self.frames_path, data)?;
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
                fs::write(&self.state_path, data)?;
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
