use anyhow::{Context, Error, Result};
use clap_derive::Args;
use inquire::Editor;
use serde_json::json;

use crate::command::Invokable;
use crate::storage::entries::Entries;
use crate::storage::entry::Entry;

#[derive(Args)]
pub(crate) struct Edit {
    hash: String,
}

impl Invokable for Edit {
    fn invoke(&self, entries: &mut Entries) -> Result<()> {
        let entry = entries.get_by_hash(self.hash.clone())?;
        let new_entry = Self::edit_entry(entry)?;
        entries.update(new_entry)
    }
}

impl Edit {
    fn edit_entry(entry: Entry) -> Result<Entry, Error> {
        let new_entry = Editor::new(&entry.get_id())
            .with_predefined_text(&json!(entry).to_string())
            .with_file_extension("json")
            .prompt()?;
        let entry: Entry = serde_json::from_str(&new_entry).context("Cannot parse given JSON")?;
        Ok(entry)
    }
}
