use std::collections::HashMap;
use std::fmt::Display;

use anyhow::{Error, Result};
use chrono::{DateTime, NaiveDate, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::commands::params::Project;
use crate::storage::entry::Entry;
use crate::storage::report::{ProjectEntry, Report};

#[derive(Serialize, Deserialize, Default)]
pub struct Entries {
    entries: Vec<Entry>,
}

impl Entries {}

impl Entries {
    pub fn push(&mut self, entry: Entry) {
        self.entries.push(entry);
    }

    pub fn get_entries(&self) -> Vec<Entry> {
        self.entries.clone()
    }
    pub fn set_last_end(&mut self, end: DateTime<Utc>) {
        let last = self.entries.last_mut();
        if let Some(last) = last {
            last.set_end(end);
        }
    }

    pub fn get_in_range(&self, from: NaiveDate, to: NaiveDate) -> Result<Self> {
        let entries = self
            .entries
            .iter()
            .filter(|entry| {
                entry.get_start().date_naive() >= from && entry.get_start().date_naive() <= to
            })
            .cloned()
            .collect();
        Ok(Entries { entries })
    }

    pub fn filter_by_project(&self, project: Project) -> Self {
        let entries = self
            .entries
            .iter()
            .filter(|entry| entry.get_project() == project)
            .cloned()
            .collect();
        Entries { entries }
    }

    pub fn report(&self) {
        let projects = self
            .entries
            .iter()
            .map(|entry| entry.get_project())
            .collect::<Vec<Project>>();

        let mut project_entries: HashMap<Project, Entries> = HashMap::new();
        for project in projects {
            let entries = self.filter_by_project(project.clone());
            project_entries.insert(project, entries);
        }

        // TODO: Own object for report
        let mut project_entries_vec: Vec<ProjectEntry> = vec![];
        for (_, entries) in project_entries {
            project_entries_vec.push(ProjectEntry::from(entries));
        }

        let report = Report::from(project_entries_vec);

        println!("{}", report);
    }

    pub fn get_unique_project(&self) -> Result<Project> {
        let projects: Vec<Project> = self
            .entries
            .iter()
            .map(|entry| entry.get_project())
            .unique()
            .collect();
        anyhow::ensure!(projects.len() == 1, "More than one project found");
        Ok(projects[0].clone())
    }

    pub fn get_last(&self) -> Option<&Entry> {
        self.entries.last()
    }

    pub(crate) fn get_by_hash(&self, hash: impl Into<String>) -> Result<Entry, Error> {
        let hash = hash.into();
        let entry = self
            .entries
            .iter()
            .find(|entry| entry.get_id().starts_with(&hash))
            .ok_or_else(|| Error::msg("Entry not found"))?;
        Ok(entry.clone())
    }

    pub(crate) fn update(&mut self, entry: Entry) -> Result<()> {
        let index = self
            .entries
            .iter()
            .position(|e| e.get_id() == entry.get_id())
            .ok_or_else(|| Error::msg("Entry not found"))?;
        self.entries[index] = entry;
        Ok(())
    }
}

impl Display for Entries {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for entry in &self.entries {
            writeln!(f, "{}", entry)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_unique_project() {
        let mut entries = Entries::default();
        let entry = Entry::new("project".into(), None, Utc::now(), None).unwrap();
        let entry2 = Entry::new("project".into(), None, Utc::now(), None).unwrap();
        entries.push(entry);
        entries.push(entry2);
        let project = entries.get_unique_project().unwrap();
        assert_eq!(project, Project::new("project"));
    }

    #[test]
    fn test_get_by_hash() {
        let mut entries = Entries::default();
        let entry = Entry::new("project".into(), None, Utc::now(), None).unwrap();
        entries.push(entry.clone());
        let found_entry = entries.get_by_hash(entry.get_id()).unwrap();
        assert_eq!(found_entry, entry);
    }

    #[test]
    fn test_get_last() {
        let mut entries = Entries::default();
        let entry = Entry::new("project".into(), None, Utc::now(), None).unwrap();
        let entry2 = Entry::new("project".into(), None, Utc::now(), None).unwrap();
        entries.push(entry.clone());
        entries.push(entry2.clone());
        let last_entry = entries.get_last().unwrap();
        assert_eq!(last_entry, &entry2);
    }

    #[test]
    fn test_filter_by_project() {
        let mut entries = Entries::default();
        let entry = Entry::new("project".into(), None, Utc::now(), None).unwrap();
        let entry2 = Entry::new("project2".into(), None, Utc::now(), None).unwrap();
        entries.push(entry.clone());
        entries.push(entry2.clone());
        let filtered_entries = entries.filter_by_project("project".into());
        assert_eq!(filtered_entries.entries.len(), 1);
        assert_eq!(filtered_entries.entries[0], entry);
    }

    #[test]
    fn test_get_in_range() {
        let mut entries = Entries::default();
        let entry = Entry::new("project".into(), None, Utc::now(), None).unwrap();
        let entry2 = Entry::new("project2".into(), None, Utc::now(), None).unwrap();
        entries.push(entry.clone());
        entries.push(entry2.clone());
        let from = Utc::now().date_naive();
        let to = Utc::now().date_naive();
        let filtered_entries = entries.get_in_range(from.into(), to).unwrap();
        assert_eq!(filtered_entries.entries.len(), 2);
    }

    #[test]
    fn test_get_entries() {
        let mut entries = Entries::default();
        let entry = Entry::new("project".into(), None, Utc::now(), None).unwrap();
        let entry2 = Entry::new("project2".into(), None, Utc::now(), None).unwrap();
        entries.push(entry.clone());
        entries.push(entry2.clone());
        let entries = entries.get_entries();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_push() {
        let mut entries = Entries::default();
        let entry = Entry::new("project".into(), None, Utc::now(), None).unwrap();
        entries.push(entry.clone());
        let entries = entries.get_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0], entry);
    }
}
