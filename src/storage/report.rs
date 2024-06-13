use std::fmt::Display;

use colored::Colorize;

use crate::storage::entries::Entries;
use crate::storage::entry::Entry;

pub(crate) struct Report {
    projects: Vec<ProjectEntry>,
}

impl From<Vec<ProjectEntry>> for Report {
    fn from(projects: Vec<ProjectEntry>) -> Self {
        Report { projects }
    }
}

impl Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for project in &self.projects {
            writeln!(f, "{}", project.name.bright_green())?;
            for entry in &project.entries {
                let report = entry
                    .print_report()
                    .unwrap_or_else(|_| format!("[{:?}]", entry.get_tags().unwrap().as_string()));
                writeln!(f, "   {}", report)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

pub(crate) struct ProjectEntry {
    name: String,
    entries: Vec<Entry>,
}

impl From<Entries> for ProjectEntry {
    fn from(entries: Entries) -> Self {
        let project = entries.get_unique_project().unwrap();
        ProjectEntry {
            name: project.to_string(),
            entries: entries.get_entries(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_from_project_entries() {
        let project_entries = vec![ProjectEntry {
            name: "default".to_string(),
            entries: vec![],
        }];
        let report = Report::from(project_entries);
        assert_eq!(report.projects.len(), 1);
    }
}
