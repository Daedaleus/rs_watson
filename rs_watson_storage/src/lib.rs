pub mod json;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameRecord {
    pub id: Uuid,
    pub project: String,
    pub tags: Vec<String>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveFrameRecord {
    pub project: String,
    pub tags: Vec<String>,
    pub start: DateTime<Utc>,
}

pub trait Storage {
    type Error: std::error::Error;

    fn load_frames(&self) -> Result<Vec<FrameRecord>, Self::Error>;
    fn save_frames(&self, frames: &[FrameRecord]) -> Result<(), Self::Error>;
    fn load_active(&self) -> Result<Option<ActiveFrameRecord>, Self::Error>;
    fn save_active(&self, frame: Option<&ActiveFrameRecord>) -> Result<(), Self::Error>;
}
