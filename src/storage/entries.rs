use std::collections::HashMap;
use std::fmt::Display;

use anyhow::{Error, Result};
use chrono::{DateTime, NaiveDate, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

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

    pub fn get_in_range(&self, from: NaiveDate, to: NaiveDate) -> anyhow::Result<Self> {
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

    pub fn filter_by_project(&self, project: String) -> Self {
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
            .collect::<Vec<String>>();

        let mut project_entries = HashMap::new();
        for project in projects {
            let entries = self.filter_by_project(project.to_string());
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

    pub fn get_unique_project(&self) -> anyhow::Result<String> {
        let projects = self
            .entries
            .iter()
            .map(|entry| entry.get_project())
            .unique()
            .collect::<Vec<String>>();
        anyhow::ensure!(projects.len() == 1, "More than one project found");
        Ok(projects[0].to_string())
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
