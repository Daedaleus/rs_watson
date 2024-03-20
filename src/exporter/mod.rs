use crate::storage::entries::Entries;

pub(crate) mod csv;

pub trait Exporter {
    fn write(entries: Entries, path: String) -> anyhow::Result<()>;
}
