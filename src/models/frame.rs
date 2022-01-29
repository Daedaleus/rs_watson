use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Frame {
    pub project: String,
    pub task: Option<String>,
    pub from: DateTime<Utc>,
    pub until: Option<DateTime<Utc>>,
}
