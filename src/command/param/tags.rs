use derive_more::{Deref, IntoIterator};
use serde::{Deserialize, Serialize};

use crate::command::param::tag::Tag;

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
                let res = s
                    .split_whitespace()
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
