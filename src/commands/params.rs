use chrono::{NaiveDate, NaiveTime};
use derive_more::{Deref, Into, IntoIterator};
use serde::{Deserialize, Serialize};

#[derive(Deref, Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Hash)]
pub(crate) struct Project(String);

impl Project {
    pub fn new(project: impl Into<String>) -> Self {
        let project = project.into();
        assert!(!project.is_empty());
        Self(project)
    }
}

impl<T> From<T> for Project
where
    T: Into<String>,
{
    fn from(value: T) -> Self {
        let value = value.into();
        Project::new(value)
    }
}

#[derive(Deref, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct Tag(String);

impl Tag {
    pub fn new(tag: String) -> Self {
        assert!(!tag.is_empty());
        Self(tag)
    }
}

impl<T> From<T> for Tag
where
    T: Into<String>,
{
    fn from(tag: T) -> Self {
        let tag = tag.into();
        Tag::new(tag)
    }
}

#[derive(Deref, Clone, Default, IntoIterator, Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct Tags(Vec<Tag>);

impl FromIterator<Tag> for Tags {
    fn from_iter<I: IntoIterator<Item = Tag>>(iter: I) -> Self {
        let tags = iter.into_iter().collect();
        Tags::new(tags)
    }
}

impl From<String> for Tags {
    fn from(s: String) -> Self {
        Tags::parse(&s).unwrap()
    }
}
impl Tags {
    pub fn new(tags: Vec<Tag>) -> Self {
        assert!(!tags.is_empty(), "Tags cannot be empty");
        Self(tags)
    }

    pub fn parse(s: &str) -> Result<Self, anyhow::Error> {
        let tags = match s.len() {
            0 => Tags::default(),
            _ => {
                let res = s.split_whitespace()
                    .map(|tag| Tag::new(tag.trim_start_matches('+').to_string()))
                    .collect();
                Tags::new(res)
            }
        };

        Ok(tags)
    }

    pub fn as_string(&self) -> String {
        self.iter()
            .map(|tag| tag.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    }
}

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
