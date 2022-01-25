use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Frame {
    pub project: String,
    pub task: Option<String>,
    #[serde(with = "chrono_datetime_as_bson_datetime")]
    pub from: Option<DateTime<Utc>>,
}