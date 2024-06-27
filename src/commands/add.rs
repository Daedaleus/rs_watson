use chrono::Local;
use clap_derive::Args;

use crate::commands::params::{At, Project, Tags};
use crate::commands::parse_time;
use crate::commands::Invokable;
use crate::storage::entries::Entries;
use crate::storage::entry::Entry;

#[derive(Args)]
pub(crate) struct Add {
    project: Project,
    tags: Tags,
    #[clap(short = 'f', value_parser = parse_time)]
    from: At,
    #[clap(short = 't', value_parser = parse_time)]
    to: At,
}

impl Invokable for Add {
    fn invoke(&self, entries: &mut Entries) -> anyhow::Result<()> {
        let now = Local::now();
        let start = now.clone().with_time(self.from.clone().into()).unwrap();
        let to = now.clone().with_time(self.to.clone().into()).unwrap();
        let entry = Entry::new(
            self.project.clone(),
            Some(self.tags.clone()),
            start.into(),
            Some(to.into()),
        )?;

        entries.push(entry);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Timelike;

    use crate::commands::params::Tag;

    use super::*;

    #[test]
    fn test_add() {
        let mut entries = Entries::default();
        let mut expected = Entries::default();
        let now = Local::now();
        let now_plus_one_minute = now.clone().with_minute(now.minute() + 1).unwrap();

        let add = Add {
            project: Project::new("test".to_string()),
            tags: Tags::new(vec![Tag::new("test".to_string())]),
            from: At::from(now.time()),
            to: At::from(now_plus_one_minute.time()),
        };
        let expected_entry = Entry::new(
            Project::new("test".to_string()),
            Some(Tags::new(vec![Tag::new("test".to_string())])),
            now.into(),
            Some(now_plus_one_minute.into()),
        )
        .unwrap();
        expected.push(expected_entry);
        add.invoke(&mut entries).unwrap();

        assert_eq!(
            expected.get_entries()[0].get_tags(),
            entries.get_entries()[0].get_tags()
        );
        assert_eq!(
            expected.get_entries()[0].get_project(),
            entries.get_entries()[0].get_project()
        );
        assert_eq!(
            expected.get_entries()[0].get_start(),
            entries.get_entries()[0].get_start()
        );
    }
}
