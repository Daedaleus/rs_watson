mod frames;
mod init;
mod io;
mod meta;
mod tracking;

use anyhow::Result;
use clap::Subcommand;
use rs_watson::Watson;
use rs_watson_storage::Storage;

use crate::time_utils::parse_date;
use rs_watson::config::{Config, WeekStart};

pub(crate) use init::cmd_init;

// ---------------------------------------------------------------------------
// CLI types
// ---------------------------------------------------------------------------

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
        /// Start time: 09:00, 2026-05-14 09:00, or "yesterday 09:00" (default: now)
        #[arg(long, value_name = "DATETIME")]
        at: Option<String>,
    },
    /// Stop the current tracking session
    Stop {
        /// Stop time: 17:30, 2026-05-14 17:30, or "yesterday 17:30" (default: now)
        #[arg(long, value_name = "DATETIME")]
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
        /// Skip the last N frames (combine with --limit for pagination)
        #[arg(long, value_name = "N")]
        offset: Option<usize>,
    },
    /// Show aggregated report for today
    Today {
        /// Group by epic instead of project (requires epics in config.toml)
        #[arg(long)]
        epic: bool,
    },
    /// Show aggregated report for all recorded time
    Report {
        /// Start date filter (YYYY-MM-DD or shortcuts: today, yesterday, week, month)
        #[arg(long, value_name = "DATE")]
        from: Option<String>,
        /// End date filter (YYYY-MM-DD or shortcuts: today, yesterday, week, month)
        #[arg(long, value_name = "DATE")]
        to: Option<String>,
        /// Group by epic instead of project (requires epics in config.toml)
        #[arg(long)]
        epic: bool,
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
        /// Start time: 09:00, 2026-05-14 09:00, or "yesterday 09:00"
        #[arg(long, value_name = "DATETIME")]
        from: String,
        /// End time: 17:30, 2026-05-14 17:30, or "yesterday 17:30"
        #[arg(long, value_name = "DATETIME")]
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
    /// List all configured epics
    Epics,
    /// Print shell completion script to stdout
    Completions {
        /// Shell to generate completions for
        shell: clap_complete::Shell,
    },
    /// Export frames to a file or stdout
    Export {
        /// Output format
        #[arg(long, value_enum, default_value = "csv")]
        format: io::ExportFormat,
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
        source: io::ImportSource,
        /// Path to the source file (default: ~/.local/share/watson/frames)
        #[arg(long, value_name = "FILE")]
        file: Option<String>,
        /// Preview what would be imported without making changes
        #[arg(long)]
        dry_run: bool,
    },
}

// ---------------------------------------------------------------------------
// Shared helpers (accessible to all submodules via `super::`)
// ---------------------------------------------------------------------------

fn w_err<E: std::fmt::Display>(e: E) -> anyhow::Error {
    anyhow::anyhow!("{e}")
}

pub(super) fn apply_date_filter(
    frames: Vec<rs_watson::Frame>,
    from: Option<String>,
    to: Option<String>,
    week_start: WeekStart,
) -> Result<Vec<rs_watson::Frame>> {
    use chrono::Local;
    let from = from.map(|s| parse_date(&s, week_start)).transpose()?;
    let to = to.map(|s| parse_date(&s, week_start)).transpose()?;
    Ok(frames
        .into_iter()
        .filter(|f| {
            let d = f.start.with_timezone(&Local).date_naive();
            from.is_none_or(|fd| d >= fd) && to.is_none_or(|td| d <= td)
        })
        .collect())
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

pub(crate) fn dispatch<S: Storage<Error: std::error::Error + Send + Sync + 'static>>(
    watson: Watson<S>,
    command: Commands,
    config: &Config,
) -> Result<()> {
    match command {
        Commands::Init | Commands::Completions { .. } => {
            unreachable!("handled before dispatch")
        }
        Commands::Start { project, tags, at } => {
            tracking::cmd_start(&watson, project, tags, at, config)
        }
        Commands::Stop { at } => tracking::cmd_stop(&watson, at, config),
        Commands::Cancel => tracking::cmd_cancel(&watson),
        Commands::Status => tracking::cmd_status(&watson),
        Commands::Log {
            from,
            to,
            limit,
            offset,
        } => frames::cmd_log(&watson, from, to, limit, offset, config),
        Commands::Today { epic } => frames::cmd_today(&watson, epic, config),
        Commands::Report { from, to, epic } => frames::cmd_report(&watson, from, to, epic, config),
        Commands::Add {
            project,
            tags,
            from,
            to,
        } => frames::cmd_add(&watson, project, tags, from, to, config),
        Commands::Edit => frames::cmd_edit(&watson),
        Commands::Remove => frames::cmd_remove(&watson),
        Commands::Rename { from, to } => meta::cmd_rename(&watson, from, to),
        Commands::Projects => meta::cmd_projects(&watson),
        Commands::Tags => meta::cmd_tags(&watson),
        Commands::Epics => meta::cmd_epics(config),
        Commands::Export {
            format,
            output,
            from,
            to,
        } => io::cmd_export(&watson, format, output, from, to, config),
        Commands::Import {
            source,
            file,
            dry_run,
        } => io::cmd_import(&watson, source, file, dry_run),
    }
}
