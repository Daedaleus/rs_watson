use derive_more::Deref;
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
