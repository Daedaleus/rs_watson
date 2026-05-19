use rs_watson::Frame;
use uuid::Uuid;

use crate::format::fmt_local_dt;

#[derive(PartialEq, Clone, Copy)]
pub(crate) enum Tab {
    Log,
    Add,
    Report,
}

pub(crate) struct EditState {
    pub(crate) id: Uuid,
    pub(crate) project: String,
    pub(crate) tags: String,
    pub(crate) start: String,
    pub(crate) end: String,
    pub(crate) error: Option<String>,
}

impl EditState {
    pub(crate) fn from_frame(f: &Frame) -> Self {
        Self {
            id: f.id,
            project: f.project.clone(),
            tags: f.tags.join(", "),
            start: fmt_local_dt(f.start),
            end: fmt_local_dt(f.end),
            error: None,
        }
    }
}
