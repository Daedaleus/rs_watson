use std::cell::RefCell;

use rs_watson_storage::{ActiveFrameRecord, FrameRecord, Storage};

/// In-memory `Storage` implementation for unit tests.
/// Uses `Infallible` as the error type — no I/O, no failure paths.
pub(crate) struct MemoryStorage {
    frames: RefCell<Vec<FrameRecord>>,
    active: RefCell<Option<ActiveFrameRecord>>,
}

impl MemoryStorage {
    pub(crate) fn new() -> Self {
        Self {
            frames: RefCell::new(Vec::new()),
            active: RefCell::new(None),
        }
    }
}

impl Storage for MemoryStorage {
    type Error = std::convert::Infallible;

    fn load_frames(&self) -> Result<Vec<FrameRecord>, Self::Error> {
        Ok(self.frames.borrow().clone())
    }
    fn save_frames(&self, frames: &[FrameRecord]) -> Result<(), Self::Error> {
        *self.frames.borrow_mut() = frames.to_vec();
        Ok(())
    }
    fn load_active(&self) -> Result<Option<ActiveFrameRecord>, Self::Error> {
        Ok(self.active.borrow().clone())
    }
    fn save_active(&self, frame: Option<&ActiveFrameRecord>) -> Result<(), Self::Error> {
        *self.active.borrow_mut() = frame.cloned();
        Ok(())
    }
}
