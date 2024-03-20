use std::collections::HashMap;
use std::fmt::Display;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::storage::entry::Entry;

#[derive(Serialize, Deserialize, Default)]
pub struct Entries {
    entries: Vec<Entry>,
}

impl Entries {
    pub fn push(&mut self, entry: Entry) {
        self.entries.push(entry);
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
        for (project, entries) in project_entries {
            println!("{}", project);
            for entry in entries.entries {
                println!("{}", entry);
            }
        }
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
