use rs_watson_storage::{ActiveFrameRecord, FrameRecord};

use crate::{ActiveFrame, Frame};

impl From<&ActiveFrame> for ActiveFrameRecord {
    fn from(f: &ActiveFrame) -> Self {
        ActiveFrameRecord {
            project: f.project.clone(),
            tags: f.tags.clone(),
            start: f.start,
        }
    }
}

impl From<ActiveFrameRecord> for ActiveFrame {
    fn from(r: ActiveFrameRecord) -> Self {
        ActiveFrame {
            project: r.project,
            tags: r.tags,
            start: r.start,
        }
    }
}

impl From<&Frame> for FrameRecord {
    fn from(f: &Frame) -> Self {
        FrameRecord {
            id: f.id,
            project: f.project.clone(),
            tags: f.tags.clone(),
            start: f.start,
            end: f.end,
        }
    }
}

impl From<FrameRecord> for Frame {
    fn from(r: FrameRecord) -> Self {
        Frame {
            id: r.id,
            project: r.project,
            tags: r.tags,
            start: r.start,
            end: r.end,
        }
    }
}
