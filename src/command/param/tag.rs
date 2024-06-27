use derive_more::Deref;
use serde::{Deserialize, Serialize};

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
