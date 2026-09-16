use rs_watson::{ActiveFrame, Frame};
use uuid::Uuid;

use crate::format::fmt_local_dt;

#[derive(PartialEq, Clone, Copy)]
pub(crate) enum Tab {
    Log,
    Add,
    Report,
}

/// What an open edit dialog is editing. The running frame has no ID — it is
/// stored separately — so it cannot be addressed like a recorded frame.
#[derive(PartialEq, Clone, Copy)]
pub(crate) enum EditTarget {
    Frame(Uuid),
    Active,
}

pub(crate) struct EditState {
    pub(crate) target: EditTarget,
    pub(crate) project: String,
    pub(crate) tags: String,
    pub(crate) start: String,
    /// Empty and unused when `target` is [`EditTarget::Active`] — it has no end yet.
    pub(crate) end: String,
    pub(crate) error: Option<String>,
}

impl EditState {
    pub(crate) fn from_frame(f: &Frame) -> Self {
        Self {
            target: EditTarget::Frame(f.id),
            project: f.project.clone(),
            tags: f.tags.join(", "),
            start: fmt_local_dt(f.start),
            end: fmt_local_dt(f.end),
            error: None,
        }
    }

    pub(crate) fn from_active(a: &ActiveFrame) -> Self {
        Self {
            target: EditTarget::Active,
            project: a.project.clone(),
            tags: a.tags.join(", "),
            start: fmt_local_dt(a.start),
            end: String::new(),
            error: None,
        }
    }
}
