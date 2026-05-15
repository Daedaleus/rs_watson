use std::collections::BTreeMap;
use std::process;

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Local, NaiveDate, NaiveTime, TimeZone, Utc};
use clap::{Parser, Subcommand};
use dialoguer::{Input, Select, theme::ColorfulTheme};
use owo_colors::OwoColorize;
use rs_watson::{Frame, Report, Watson};
use rs_watson_storage::json::JsonStorage;

#[derive(Parser)]
#[command(name = "watson", about = "Time tracking tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
    Log,
    /// Show aggregated report for today
    Today,
    /// Show aggregated report for all recorded time
    Report,
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
    /// List all projects that have been tracked
    Projects,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {}", "Error:".red().bold(), e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    let data_dir = dirs::data_dir()
        .context("Could not determine data directory")?
        .join("rs_watson");
    std::fs::create_dir_all(&data_dir)
        .with_context(|| format!("Could not create data directory: {}", data_dir.display()))?;

    let storage = JsonStorage::new(&data_dir);
    let watson = Watson::new(storage);

    match cli.command {
        Commands::Start { project, tags, at } => {
            let time = at.map(|s| parse_at(&s)).transpose()?.unwrap_or_else(Utc::now);
            let frame = watson
                .start(&project, tags, time)
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            println!(
                "{} {}{}  {}",
                "Starting".green().bold(),
                frame.project.yellow().bold(),
                fmt_tags(&frame.tags),
                fmt_time(frame.start).bright_black(),
            );
        }
        Commands::Stop { at } => {
            let time = at.map(|s| parse_at(&s)).transpose()?.unwrap_or_else(Utc::now);
            let frame = watson
                .stop(time)
                .map_err(|e| anyhow::anyhow!("{e}"))?;

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
            println!("  {}", fmt_duration(frame.end - frame.start).magenta().bold());
        }
        Commands::Cancel => {
            let frame = watson
                .cancel()
                .map_err(|e| anyhow::anyhow!("{e}"))?;

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
            match watson.status().map_err(|e| anyhow::anyhow!("{e}"))? {
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
        Commands::Log => {
            let frames = watson.log().map_err(|e| anyhow::anyhow!("{e}"))?;
            if frames.is_empty() {
                println!("{}", "No frames recorded yet.".bright_black());
            } else {
                print_frames_grouped(&frames);
            }
        }
        Commands::Today => {
            let today = Local::now().date_naive();
            let frames: Vec<Frame> = watson
                .log()
                .map_err(|e| anyhow::anyhow!("{e}"))?
                .into_iter()
                .filter(|f| f.start.with_timezone(&Local).date_naive() == today)
                .collect();
            if frames.is_empty() {
                println!("{}", "No frames recorded today.".bright_black());
            } else {
                print_report_grouped(&frames, false);
            }
        }
        Commands::Edit => {
            let frames = watson.log().map_err(|e| anyhow::anyhow!("{e}"))?;
            if frames.is_empty() {
                println!("{}", "No frames to edit.".bright_black());
                return Ok(());
            }

            // Build display items for the selector
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
            let new_end   = prompt_time("End   (HH:MM or HH:MM:SS)", frame.end)?;

            if new_end <= new_start {
                anyhow::bail!("End time must be after start time");
            }

            let updated = watson
                .edit(frame.id, new_project, new_tags, new_start, new_end)
                .map_err(|e| anyhow::anyhow!("{e}"))?;

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
            println!("  {}", fmt_duration(updated.end - updated.start).magenta().bold());
        }
        Commands::Add { project, tags, from, to } => {
            let start = parse_at(&from)?;
            let end   = parse_at(&to)?;
            let frame = watson
                .add(&project, tags, start, end)
                .map_err(|e| anyhow::anyhow!("{e}"))?;

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
            println!("  {}", fmt_duration(frame.end - frame.start).magenta().bold());
        }
        Commands::Projects => {
            let projects = watson.projects().map_err(|e| anyhow::anyhow!("{e}"))?;
            if projects.is_empty() {
                println!("{}", "No projects recorded yet.".bright_black());
            } else {
                for name in &projects {
                    println!("{}", name.yellow().bold());
                }
            }
        }
        Commands::Report => {
            let frames = watson.log().map_err(|e| anyhow::anyhow!("{e}"))?;
            if frames.is_empty() {
                println!("{}", "No frames recorded yet.".bright_black());
            } else {
                print_report_grouped(&frames, true);
            }
        }
    }

    Ok(())
}

/// Prints frames as individual entries grouped by day (used by `log`).
fn print_frames_grouped(frames: &[Frame]) {
    let mut by_day: BTreeMap<NaiveDate, Vec<&Frame>> = BTreeMap::new();
    for frame in frames {
        by_day
            .entry(frame.start.with_timezone(&Local).date_naive())
            .or_default()
            .push(frame);
    }

    for (date, day_frames) in &by_day {
        let total = day_frames
            .iter()
            .fold(Duration::zero(), |acc, f| acc + (f.end - f.start));

        println!(
            "{}  {}",
            date.format("%A, %d %B %Y").to_string().bold().white(),
            format!("({})", fmt_duration(total)).bright_black(),
        );

        for frame in day_frames {
            println!(
                "  {}  {}  {}   {:<12}  {}{}",
                fmt_time(frame.start).bright_white(),
                "→".white(),
                fmt_time(frame.end).bright_white(),
                fmt_duration(frame.end - frame.start).magenta().bold(),
                frame.project.yellow().bold(),
                fmt_tags(&frame.tags),
            );
        }
        println!();
    }
}

/// Prints frames as an aggregated report grouped by day (used by `today` and `report`).
/// Shows grand total when `show_total` is true and there is more than one day.
fn print_report_grouped(frames: &[Frame], show_total: bool) {
    let mut by_day: BTreeMap<NaiveDate, Vec<&Frame>> = BTreeMap::new();
    for frame in frames {
        by_day
            .entry(frame.start.with_timezone(&Local).date_naive())
            .or_default()
            .push(frame);
    }

    let grand_total = frames
        .iter()
        .fold(Duration::zero(), |acc, f| acc + (f.end - f.start));

    for (date, day_frames) in &by_day {
        let owned: Vec<Frame> = day_frames.iter().map(|f| (*f).clone()).collect();
        let report = Report::from_frames(&owned);

        println!(
            "{}  {}",
            date.format("%A, %d %B %Y").to_string().bold().white(),
            format!("({})", fmt_duration(report.total)).bright_black(),
        );
        println!();

        for project in &report.projects {
            println!(
                "  {}  {}",
                format!("{:<20}", project.name).yellow().bold(),
                fmt_duration(project.total).magenta().bold(),
            );
            for tag in &project.tags {
                println!(
                    "    {}  {}",
                    format!("{:<18}", tag.name).cyan(),
                    fmt_duration(tag.total).magenta(),
                );
            }
        }
        println!();
    }

    if show_total && by_day.len() > 1 {
        println!(
            "{}  {}",
            "Total".bold().white(),
            fmt_duration(grand_total).magenta().bold(),
        );
    }
}

/// Parses a local time string (HH:MM or HH:MM:SS) relative to today and returns UTC.
fn parse_at(input: &str) -> Result<DateTime<Utc>> {
    let local_now = Local::now();
    let time = NaiveTime::parse_from_str(input.trim(), "%H:%M:%S")
        .or_else(|_| NaiveTime::parse_from_str(input.trim(), "%H:%M"))
        .with_context(|| format!("Invalid time \"{input}\", expected HH:MM or HH:MM:SS"))?;
    let naive_local = local_now.date_naive().and_time(time);
    Local
        .from_local_datetime(&naive_local)
        .single()
        .map(|dt| dt.with_timezone(&Utc))
        .context("Ambiguous time (DST transition)")
}

/// Prompts for a time value in local time, pre-filled with the local representation of `default`.
/// Parses the input as local time on the same local date, then converts back to UTC.
fn prompt_time(prompt: &str, default: DateTime<Utc>) -> Result<DateTime<Utc>> {
    let local_default = default.with_timezone(&Local);
    loop {
        let input = Input::<String>::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .with_initial_text(local_default.format("%H:%M:%S").to_string())
            .interact_text()?;

        let parsed = NaiveTime::parse_from_str(input.trim(), "%H:%M:%S")
            .or_else(|_| NaiveTime::parse_from_str(input.trim(), "%H:%M"));

        match parsed {
            Ok(t) => {
                let naive_local = local_default.date_naive().and_time(t);
                match Local.from_local_datetime(&naive_local).single() {
                    Some(local_dt) => return Ok(local_dt.with_timezone(&Utc)),
                    None => eprintln!("  {} Ambiguous time (DST transition), try again.", "Warning:".yellow()),
                }
            }
            Err(_) => eprintln!("  {} Use HH:MM or HH:MM:SS", "Invalid time.".red()),
        }
    }
}

fn fmt_tags(tags: &[String]) -> String {
    if tags.is_empty() {
        String::new()
    } else {
        format!("  {}", format!("[{}]", tags.join(", ")).cyan())
    }
}

fn fmt_time(dt: DateTime<Utc>) -> String {
    dt.with_timezone(&Local).format("%H:%M:%S").to_string()
}

fn fmt_duration(d: Duration) -> String {
    let total = d.num_seconds().max(0);
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;

    if h > 0 {
        format!("{}h {}m {}s", h, m, s)
    } else if m > 0 {
        format!("{}m {}s", m, s)
    } else {
        format!("{}s", s)
    }
}
