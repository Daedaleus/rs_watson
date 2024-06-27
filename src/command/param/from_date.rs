use chrono::NaiveDate;
use derive_more::Deref;

#[derive(Deref, Clone, PartialEq, PartialOrd)]
pub struct FromDate(Option<NaiveDate>);

impl From<Option<NaiveDate>> for FromDate {
    fn from(from: Option<NaiveDate>) -> Self {
        FromDate(from)
    }
}

impl From<NaiveDate> for FromDate {
    fn from(from: NaiveDate) -> Self {
        FromDate(Some(from))
    }
}

impl FromDate {
    pub fn or_min(&self) -> NaiveDate {
        self.unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap())
    }
}
