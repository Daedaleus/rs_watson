use chrono::NaiveTime;
use derive_more::Deref;

#[derive(Deref, Clone)]
pub struct At(NaiveTime);

impl From<NaiveTime> for At {
    fn from(at: NaiveTime) -> Self {
        At(at)
    }
}

#[allow(clippy::from_over_into)]
impl Into<NaiveTime> for At {
    fn into(self) -> NaiveTime {
        self.0
    }
}
