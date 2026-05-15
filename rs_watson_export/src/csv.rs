use std::io::Write;

use rs_watson::Frame;
use thiserror::Error;

use crate::Exporter;

#[derive(Debug, Error)]
pub enum CsvExportError {
    #[error("CSV error: {0}")]
    Csv(#[from] ::csv::Error),
}

pub struct CsvExporter;

impl Exporter for CsvExporter {
    type Error = CsvExportError;

    fn export<W: Write>(&self, frames: &[Frame], writer: W) -> Result<(), Self::Error> {
        let mut wtr = ::csv::Writer::from_writer(writer);

        wtr.write_record(["id", "project", "tags", "start", "end", "duration_seconds"])?;

        for frame in frames {
            wtr.write_record([
                frame.id.to_string(),
                frame.project.clone(),
                frame.tags.join("|"),
                frame
                    .start
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                frame.end.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                (frame.end - frame.start).num_seconds().to_string(),
            ])?;
        }

        wtr.into_inner()
            .map_err(|e| CsvExportError::Csv(e.into_error().into()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    fn frame(project: &str, tags: &[&str], h_start: u32, h_end: u32) -> Frame {
        Frame {
            id: Uuid::nil(),
            project: project.into(),
            tags: tags.iter().map(|s| s.to_string()).collect(),
            start: Utc.with_ymd_and_hms(2026, 1, 15, h_start, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2026, 1, 15, h_end, 0, 0).unwrap(),
        }
    }

    #[test]
    fn csv_export_produces_header_and_rows() {
        let frames = vec![frame("backend", &["api", "auth"], 9, 10)];
        let mut buf = Vec::new();
        CsvExporter.export(&frames, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.starts_with("id,project,tags,start,end,duration_seconds\n"));
        assert!(output.contains("backend"));
        assert!(output.contains("api|auth"));
        assert!(output.contains("3600")); // 1 hour in seconds
    }

    #[test]
    fn csv_export_empty_tags_produces_empty_field() {
        let frames = vec![frame("backend", &[], 9, 10)];
        let mut buf = Vec::new();
        CsvExporter.export(&frames, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        // tags column should be empty
        assert!(output.contains("backend,,"));
    }
}
