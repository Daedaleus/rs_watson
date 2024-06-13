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
        let tags = s
            .split_whitespace()
            .map(|tag| Tag::new(tag.trim_start_matches('+').to_string()))
            .collect();
        Ok(Tags::new(tags))
    }

    pub fn as_string(&self) -> String {
        self.iter()
            .map(|tag| tag.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    }
}
