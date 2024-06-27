use chrono::NaiveDate;
use derive_more::Deref;

#[derive(Deref, Clone, PartialEq, PartialOrd)]
pub struct ToDate(Option<NaiveDate>);

impl From<Option<NaiveDate>> for ToDate {
    fn from(to: Option<NaiveDate>) -> Self {
        ToDate(to)
    }
}

impl From<NaiveDate> for ToDate {
    fn from(to: NaiveDate) -> Self {
        ToDate(Some(to))
    }
}

impl ToDate {
    pub fn or_max(&self) -> NaiveDate {
        self.unwrap_or_else(|| NaiveDate::from_ymd_opt(9999, 12, 31).unwrap())
    }
}
