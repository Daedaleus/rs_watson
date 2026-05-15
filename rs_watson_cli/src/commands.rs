use std::fs;
use std::io;

use anyhow::{Context, Result};
use chrono::Utc;
use clap::Subcommand;
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use owo_colors::OwoColorize;
use rs_watson::Watson;
use rs_watson_storage::Storage;

use crate::config::{BehaviorConfig, Config, StorageConfig, StorageProvider};
use crate::format::{fmt_duration, fmt_tags, fmt_time, print_frames_grouped, print_report_grouped};
use crate::time_utils::{check_future, parse_at, parse_date, prompt_time};

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Create the config file interactively
    Init,
    /// Start tracking time on a project
    Start {
        /// Project name
        #[arg(short = 'p', long)]
        project: String,
        /// Tags (can be specified multiple times)
        #[arg(short = 't', long = "tag")]
        tags: Vec<String>,
        /// Start time in local time, e.g. 09:00 or 09:00:00 (default: now)
        #[arg(long, value_name = "HH:MM")]
        at: Option<String>,
    },
    /// Stop the current tracking session
    Stop {
        /// Stop time in local time, e.g. 17:30 or 17:30:00 (default: now)
        #[arg(long, value_name = "HH:MM")]
        at: Option<String>,
    },
    /// Cancel the current tracking session without saving
    Cancel,
    /// Show what is currently being tracked
    Status,
    /// List completed frames, grouped by day
    Log {
        /// Start date filter (YYYY-MM-DD or shortcuts: today, yesterday, week, month)
        #[arg(long, value_name = "DATE")]
        from: Option<String>,
        /// End date filter (YYYY-MM-DD or shortcuts: today, yesterday, week, month)
        #[arg(long, value_name = "DATE")]
        to: Option<String>,
        /// Show only the last N frames
        #[arg(long, value_name = "N")]
        limit: Option<usize>,
    },
    /// Show aggregated report for today
    Today,
    /// Show aggregated report for all recorded time
    Report {
        /// Start date filter (YYYY-MM-DD or shortcuts: today, yesterday, week, month)
        #[arg(long, value_name = "DATE")]
        from: Option<String>,
        /// End date filter (YYYY-MM-DD or shortcuts: today, yesterday, week, month)
        #[arg(long, value_name = "DATE")]
        to: Option<String>,
    },
    /// Edit a recorded frame interactively
    Edit,
    /// Add a completed frame retroactively
    Add {
        /// Project name
        #[arg(short = 'p', long)]
        project: String,
        /// Tags (can be specified multiple times)
        #[arg(short = 't', long = "tag")]
        tags: Vec<String>,
        /// Start time in local time, e.g. 09:00 or 09:00:00
        #[arg(long, value_name = "HH:MM")]
        from: String,
        /// End time in local time, e.g. 17:30 or 17:30:00
        #[arg(long, value_name = "HH:MM")]
        to: String,
    },
    /// Remove a recorded frame interactively
    Remove,
    /// Rename a project across all recorded frames
    Rename {
        /// Current project name
        from: String,
        /// New project name
        to: String,
    },
    /// List all tags that have been used
    Tags,
    /// List all projects that have been tracked
    Projects,
    /// Print shell completion script to stdout
    Completions {
        /// Shell to generate completions for
        shell: clap_complete::Shell,
    },
    /// Export frames to a file or stdout
    Export {
        /// Output format
        #[arg(long, value_enum, default_value = "csv")]
        format: ExportFormat,
        /// Output file (default: stdout)
        #[arg(long, value_name = "FILE")]
        output: Option<String>,
        /// Start date filter
        #[arg(long, value_name = "DATE")]
        from: Option<String>,
        /// End date filter
        #[arg(long, value_name = "DATE")]
        to: Option<String>,
    },
    /// Import frames from an external source
    Import {
        /// Source format
        #[arg(long, value_enum, default_value = "watson")]
        source: ImportSource,
        /// Path to the source file (default: ~/.local/share/watson/frames)
        #[arg(long, value_name = "FILE")]
        file: Option<String>,
    },
}

#[derive(clap::ValueEnum, Clone, Copy)]
pub(crate) enum ExportFormat {
    Csv,
}

#[derive(clap::ValueEnum, Clone, Copy)]
pub(crate) enum ImportSource {
    Watson,
}

/// Converts a Watson error into an anyhow error.
fn w_err<E: std::fmt::Display>(e: E) -> anyhow::Error {
    anyhow::anyhow!("{e}")
}

/// Filters frames to those whose local start date falls within [from, to] (both inclusive).
fn apply_date_filter(
    frames: Vec<rs_watson::Frame>,
    from: Option<String>,
    to: Option<String>,
) -> Result<Vec<rs_watson::Frame>> {
    use chrono::Local;
    let from = from.map(|s| parse_date(&s)).transpose()?;
    let to = to.map(|s| parse_date(&s)).transpose()?;
    Ok(frames
        .into_iter()
        .filter(|f| {
            let d = f.start.with_timezone(&Local).date_naive();
            from.is_none_or(|fd| d >= fd) && to.is_none_or(|td| d <= td)
        })
        .collect())
}

pub(crate) fn dispatch<S>(watson: Watson<S>, command: Commands, config: &Config) -> Result<()>
where
    S: Storage,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    match command {
        Commands::Init | Commands::Completions { .. } => {
            unreachable!("handled before dispatch")
        }

        Commands::Start { project, tags, at } => {
            let time = at
                .map(|s| parse_at(&s))
                .transpose()?
                .unwrap_or_else(Utc::now);
            check_future(time, config)?;
            let frame = watson.start(&project, tags, time).map_err(w_err)?;
            println!(
                "{} {}{}  {}",
                "Starting".green().bold(),
                frame.project.yellow().bold(),
                fmt_tags(&frame.tags),
                fmt_time(frame.start).bright_black(),
            );
        }

        Commands::Stop { at } => {
            let time = at
                .map(|s| parse_at(&s))
                .transpose()?
                .unwrap_or_else(Utc::now);
            check_future(time, config)?;
            let frame = watson.stop(time).map_err(w_err)?;
            println!(
                "{} {}{}",
                "Stopped ".red().bold(),
                frame.project.yellow().bold(),
                fmt_tags(&frame.tags),
            );
            println!(
                "  {}  {}  {}",
                fmt_time(frame.start).bright_white(),
                "→".white(),
                fmt_time(frame.end).bright_white(),
            );
            println!(
                "  {}",
                fmt_duration(frame.end - frame.start).magenta().bold()
            );
        }

        Commands::Cancel => {
            let frame = watson.cancel().map_err(w_err)?;
            let discarded = fmt_duration(Utc::now() - frame.start);
            println!(
                "{} {}{}",
                "Cancelled".red().bold(),
                frame.project.yellow().bold(),
                fmt_tags(&frame.tags),
            );
            println!(
                "  {} {}  {} {}",
                "started".bright_black(),
                fmt_time(frame.start).bright_white(),
                discarded.magenta().bold(),
                "discarded".bright_black(),
            );
        }

        Commands::Status => {
            let now = Utc::now();
            match watson.status().map_err(w_err)? {
                Some(frame) => {
                    println!(
                        "{} {}{}",
                        "Tracking".blue().bold(),
                        frame.project.yellow().bold(),
                        fmt_tags(&frame.tags),
                    );
                    println!(
                        "  {} {}  {}",
                        "since".bright_black(),
                        fmt_time(frame.start).bright_white(),
                        fmt_duration(now - frame.start).magenta().bold(),
                    );
                }
                None => println!("{}", "Not tracking anything.".bright_black()),
            }
        }

        Commands::Log { from, to, limit } => {
            let mut frames = apply_date_filter(watson.log().map_err(w_err)?, from, to)?;
            if let Some(n) = limit {
                let skip = frames.len().saturating_sub(n);
                frames = frames.into_iter().skip(skip).collect();
            }
            if frames.is_empty() {
                println!("{}", "No frames recorded.".bright_black());
            } else {
                print_frames_grouped(&frames);
            }
        }

        Commands::Today => {
            use chrono::Local;
            let today = Local::now().date_naive();
            let frames: Vec<_> = watson
                .log()
                .map_err(w_err)?
                .into_iter()
                .filter(|f| f.start.with_timezone(&Local).date_naive() == today)
                .collect();
            if frames.is_empty() {
                println!("{}", "No frames recorded today.".bright_black());
            } else {
                print_report_grouped(&frames, false);
            }
        }

        Commands::Report { from, to } => {
            let frames = apply_date_filter(watson.log().map_err(w_err)?, from, to)?;
            if frames.is_empty() {
                println!("{}", "No frames recorded.".bright_black());
            } else {
                print_report_grouped(&frames, true);
            }
        }

        Commands::Edit => {
            let frames = watson.log().map_err(w_err)?;
            if frames.is_empty() {
                println!("{}", "No frames to edit.".bright_black());
                return Ok(());
            }

            let items: Vec<String> = frames
                .iter()
                .map(|f| {
                    format!(
                        "{}  {} → {}  {:<10}  {}{}",
                        f.start.format("%Y-%m-%d"),
                        fmt_time(f.start),
                        fmt_time(f.end),
                        fmt_duration(f.end - f.start),
                        f.project,
                        if f.tags.is_empty() {
                            String::new()
                        } else {
                            format!("  [{}]", f.tags.join(", "))
                        },
                    )
                })
                .collect();

            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select frame to edit")
                .items(&items)
                .default(items.len() - 1)
                .interact()?;

            let frame = &frames[selection];
            println!();

            let new_project: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Project")
                .with_initial_text(&frame.project)
                .interact_text()?;

            let tags_default = frame.tags.join(", ");
            let tags_input: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Tags  (comma-separated, empty for none)")
                .with_initial_text(&tags_default)
                .interact_text()?;
            let new_tags: Vec<String> = tags_input
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            let new_start = prompt_time("Start (HH:MM or HH:MM:SS)", frame.start)?;
            let new_end = prompt_time("End   (HH:MM or HH:MM:SS)", frame.end)?;

            if new_end <= new_start {
                anyhow::bail!("End time must be after start time");
            }

            let updated = watson
                .edit(frame.id, new_project, new_tags, new_start, new_end)
                .map_err(w_err)?;

            println!();
            println!(
                "{} {}{}",
                "Updated ".green().bold(),
                updated.project.yellow().bold(),
                fmt_tags(&updated.tags),
            );
            println!(
                "  {}  {}  {}",
                fmt_time(updated.start).bright_white(),
                "→".white(),
                fmt_time(updated.end).bright_white(),
            );
            println!(
                "  {}",
                fmt_duration(updated.end - updated.start).magenta().bold()
            );
        }

        Commands::Add {
            project,
            tags,
            from,
            to,
        } => {
            let start = parse_at(&from)?;
            let end = parse_at(&to)?;
            check_future(start, config)?;
            check_future(end, config)?;
            let frame = watson.add(&project, tags, start, end).map_err(w_err)?;
            println!(
                "{} {}{}",
                "Added   ".green().bold(),
                frame.project.yellow().bold(),
                fmt_tags(&frame.tags),
            );
            println!(
                "  {}  {}  {}",
                fmt_time(frame.start).bright_white(),
                "→".white(),
                fmt_time(frame.end).bright_white(),
            );
            println!(
                "  {}",
                fmt_duration(frame.end - frame.start).magenta().bold()
            );
        }

        Commands::Remove => {
            let frames = watson.log().map_err(w_err)?;
            if frames.is_empty() {
                println!("{}", "No frames to remove.".bright_black());
                return Ok(());
            }

            let items: Vec<String> = frames
                .iter()
                .map(|f| {
                    format!(
                        "{}  {} → {}  {:<10}  {}{}",
                        f.start.format("%Y-%m-%d"),
                        fmt_time(f.start),
                        fmt_time(f.end),
                        fmt_duration(f.end - f.start),
                        f.project,
                        if f.tags.is_empty() {
                            String::new()
                        } else {
                            format!("  [{}]", f.tags.join(", "))
                        },
                    )
                })
                .collect();

            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select frame to remove")
                .items(&items)
                .default(items.len() - 1)
                .interact()?;

            let frame = &frames[selection];

            let confirmed = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(format!(
                    "Remove \"{}\" ({} → {})?",
                    frame.project,
                    fmt_time(frame.start),
                    fmt_time(frame.end),
                ))
                .default(false)
                .interact()?;

            if !confirmed {
                println!("{}", "Aborted.".bright_black());
                return Ok(());
            }

            watson.remove(frame.id).map_err(w_err)?;
            println!(
                "{} {}{}",
                "Removed ".red().bold(),
                frame.project.yellow().bold(),
                fmt_tags(&frame.tags),
            );
        }

        Commands::Rename { from, to } => {
            let count = watson.rename(&from, &to).map_err(w_err)?;
            println!(
                "{}  {}  {}  {} {}",
                "Renamed".green().bold(),
                from.yellow().bold(),
                "→".white(),
                to.yellow().bold(),
                format!("({count} updated)").bright_black(),
            );
        }

        Commands::Tags => {
            let tags = watson.tags().map_err(w_err)?;
            if tags.is_empty() {
                println!("{}", "No tags recorded yet.".bright_black());
            } else {
                for tag in &tags {
                    println!("{}", tag.cyan());
                }
            }
        }

        Commands::Projects => {
            let projects = watson.projects().map_err(w_err)?;
            if projects.is_empty() {
                println!("{}", "No projects recorded yet.".bright_black());
            } else {
                for name in &projects {
                    println!("{}", name.yellow().bold());
                }
            }
        }

        Commands::Export {
            format,
            output,
            from,
            to,
        } => {
            use rs_watson_export::Exporter;
            use rs_watson_export::csv::CsvExporter;

            let frames = apply_date_filter(watson.log().map_err(w_err)?, from, to)?;
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
        }

        Commands::Import { source, file } => {
            let path = match (source, file) {
                (ImportSource::Watson, Some(p)) => std::path::PathBuf::from(p),
                (ImportSource::Watson, None) => dirs::data_dir()
                    .context("Could not determine data directory")?
                    .join("watson")
                    .join("frames"),
            };

            let content = fs::read_to_string(&path)
                .with_context(|| format!("Could not read: {}", path.display()))?;

            let frames = parse_watson_frames(&content)?;
            let count = frames.len();
            watson.import_frames(frames).map_err(w_err)?;

            println!(
                "{} {} {}",
                "Imported".green().bold(),
                count.to_string().yellow().bold(),
                "frames from Watson.".bright_black(),
            );
        }
    }

    Ok(())
}

/// Parses the original Watson frames file format.
/// Each frame is stored as: [id, start_ts, stop_ts, project, updated_ts, [tags]]
fn parse_watson_frames(content: &str) -> Result<Vec<rs_watson::Frame>> {
    use chrono::{TimeZone, Utc};

    let raw: Vec<serde_json::Value> =
        serde_json::from_str(content).context("Invalid Watson frames file")?;

    raw.into_iter()
        .enumerate()
        .map(|(i, entry)| {
            let ctx = || format!("Frame #{i}");
            let arr = entry.as_array().with_context(ctx)?;
            let start_ts = arr.get(1).and_then(|v| v.as_i64()).with_context(ctx)?;
            let stop_ts = arr.get(2).and_then(|v| v.as_i64()).with_context(ctx)?;
            let project = arr
                .get(3)
                .and_then(|v| v.as_str())
                .with_context(ctx)?
                .to_string();
            let tags: Vec<String> = arr
                .get(5)
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

pub(crate) fn cmd_init() -> Result<()> {
    let config_dir = dirs::config_dir()
        .context("Could not determine config directory")?
        .join("rs_watson");
    let config_path = config_dir.join("config.toml");

    if config_path.exists() {
        let overwrite = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Config file already exists. Overwrite?")
            .default(false)
            .interact()?;
        if !overwrite {
            println!("{}", "Aborted.".bright_black());
            return Ok(());
        }
    }

    println!();

    let provider_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Storage provider")
        .items(["JSON  (frames.json + state.json)", "SQLite  (watson.db)"])
        .default(0)
        .interact()?;
    let provider = match provider_idx {
        1 => StorageProvider::Sqlite,
        _ => StorageProvider::Json,
    };

    println!();

    let allow_future_times = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Allow future times for start, stop and add?")
        .default(false)
        .interact()?;

    let config = Config {
        storage: StorageConfig { provider },
        behavior: BehaviorConfig { allow_future_times },
    };

    std::fs::create_dir_all(&config_dir).with_context(|| {
        format!(
            "Could not create config directory: {}",
            config_dir.display()
        )
    })?;

    let content = toml::to_string(&config).context("Could not serialize config")?;
    std::fs::write(&config_path, &content)
        .with_context(|| format!("Could not write config: {}", config_path.display()))?;

    println!();
    println!(
        "{} {}",
        "Config written to".green().bold(),
        config_path.display().to_string().bright_white(),
    );
    println!();
    for line in content.lines() {
        println!("  {}", line.bright_black());
    }

    Ok(())
}
