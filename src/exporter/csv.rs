use crate::exporter::Exporter;
use crate::storage::entries::Entries;

pub struct CsvExporter;
impl Exporter for CsvExporter {
    fn write(entries: Entries, path: String) -> anyhow::Result<()> {
        let mut writer = csv::Writer::from_path(path)?;
        for entry in entries.get_entries() {
            writer.serialize(entry)?;
        }
        writer.flush()?;
        Ok(())
    }
}
