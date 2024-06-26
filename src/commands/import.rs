use anyhow::bail;
use clap_derive::Args;

use crate::commands::Invokable;
use crate::importer::ts_watson::TdWatsonFrame;
use crate::storage::entries::Entries;
use crate::storage::entry::Entry;

#[derive(Args)]
pub(crate) struct Import {
    from: String,
}

impl Invokable for Import {
    fn invoke(&self, entries: &mut Entries) -> anyhow::Result<()> {
        if !entries.get_entries().is_empty() {
            bail!("You already have entries!")
        }
        let imported = TdWatsonFrame::from_file(&self.from)?;
        for frame in imported {
            entries.push(Entry::from(frame));
        }
        Ok(())
    }
}
