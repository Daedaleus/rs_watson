use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Frame {
    pub project: String,
    pub task: Option<String>,
}