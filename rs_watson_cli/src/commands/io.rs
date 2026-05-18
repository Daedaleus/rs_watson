use std::fs;
use std::io;

use anyhow::{Context, Result};
use chrono::TimeZone;
use owo_colors::OwoColorize;
use rs_watson::Watson;
use rs_watson_storage::Storage;

use crate::format::{fmt_tags, fmt_time};
use rs_watson::config::Config;

use super::{apply_date_filter, w_err};

#[derive(clap::ValueEnum, Clone, Copy)]
pub(crate) enum ExportFormat {
    Csv,
}

#[derive(clap::ValueEnum, Clone, Copy)]
pub(crate) enum ImportSource {
    Watson,
}

pub(super) fn cmd_export<S: Storage<Error: std::error::Error + Send + Sync + 'static>>(
    watson: &Watson<S>,
    format: ExportFormat,
    output: Option<String>,
    from: Option<String>,
    to: Option<String>,
    config: &Config,
) -> Result<()> {
    use rs_watson_export::Exporter;
    use rs_watson_export::csv::CsvExporter;

    let frames = apply_date_filter(
        watson.log().map_err(w_err)?,
        from,
        to,
        config.behavior.week_start,
    )?;
    if frames.is_empty() {
        println!("{}", "No frames to export.".bright_black());
        return Ok(());
    }

    match format {
        ExportFormat::Csv => match output {
            Some(path) => {
                let file = fs::File::create(&path)
                    .with_context(|| format!("Could not create file: {path}"))?;
                CsvExporter.export(&frames, file).context("Export failed")?;
                println!(
                    "{} {} {} {}",
                    "Exported".green().bold(),
                    frames.len().to_string().yellow().bold(),
                    "frames to".bright_black(),
                    path.bright_white(),
                );
            }
            None => {
                CsvExporter
                    .export(&frames, io::stdout())
                    .context("Export failed")?;
            }
        },
    }
    Ok(())
}

pub(super) fn cmd_import<S: Storage<Error: std::error::Error + Send + Sync + 'static>>(
    watson: &Watson<S>,
    source: ImportSource,
    file: Option<String>,
    dry_run: bool,
) -> Result<()> {
    let path = match (source, file) {
        (ImportSource::Watson, Some(p)) => std::path::PathBuf::from(p),
        (ImportSource::Watson, None) => dirs::data_dir()
            .context("Could not determine data directory")?
            .join("watson")
            .join("frames"),
    };

    let content =
        fs::read_to_string(&path).with_context(|| format!("Could not read: {}", path.display()))?;

    let frames = parse_watson_frames(&content)?;
    let count = frames.len();

    if dry_run {
        println!(
            "{} {} {} {}",
            "Would import".bright_black(),
            count.to_string().yellow().bold(),
            "frames".bright_black(),
            "(dry run — no changes made)".bright_black(),
        );
        for frame in &frames {
            println!(
                "  {}  {}  {}{}",
                fmt_time(frame.start).bright_white(),
                fmt_time(frame.end).bright_white(),
                frame.project.yellow().bold(),
                fmt_tags(&frame.tags),
            );
        }
    } else {
        watson.import_frames(frames).map_err(w_err)?;
        println!(
            "{} {} {}",
            "Imported".green().bold(),
            count.to_string().yellow().bold(),
            "frames from Watson.".bright_black(),
        );
    }
    Ok(())
}

/// Parses the original Watson frames file format.
/// Each frame is stored as: [start_ts, stop_ts, project, id, [tags], updated_ts]
fn parse_watson_frames(content: &str) -> Result<Vec<rs_watson::Frame>> {
    use chrono::Utc;

    let raw: Vec<serde_json::Value> =
        serde_json::from_str(content).context("Invalid Watson frames file")?;

    raw.into_iter()
        .enumerate()
        .map(|(i, entry)| {
            let ctx = || format!("Frame #{i}");
            let arr = entry.as_array().with_context(ctx)?;
            let start_ts = arr.first().and_then(|v| v.as_i64()).with_context(ctx)?;
            let stop_ts = arr.get(1).and_then(|v| v.as_i64()).with_context(ctx)?;
            let project = arr
                .get(2)
                .and_then(|v| v.as_str())
                .with_context(ctx)?
                .to_string();
            let tags: Vec<String> = arr
                .get(4)
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|t| t.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            let start = Utc.timestamp_opt(start_ts, 0).single().with_context(ctx)?;
            let end = Utc.timestamp_opt(stop_ts, 0).single().with_context(ctx)?;
            Ok(rs_watson::Frame::new(project, tags, start, end))
        })
        .collect()
}
