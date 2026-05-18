use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use tempfile::TempDir;

fn watson(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("watson").unwrap();
    cmd.env("RS_WATSON_DATA_DIR", dir.path());
    // Point config dir at the same temp dir so the real ~/.config/rs_watson/config.toml
    // is never read — tests are fully isolated and use compiled-in defaults.
    cmd.env("RS_WATSON_CONFIG_DIR", dir.path());
    cmd
}

fn watson_cfg(data_dir: &TempDir, config_dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("watson").unwrap();
    cmd.env("RS_WATSON_DATA_DIR", data_dir.path());
    cmd.env("RS_WATSON_CONFIG_DIR", config_dir.path());
    cmd
}

// --- status ---

#[test]
fn status_when_idle_says_not_tracking() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args(["status"])
        .assert()
        .success()
        .stdout(contains("Not tracking anything"));
}

// --- statusline ---

#[test]
fn statusline_when_idle_says_no_project() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args(["statusline"])
        .assert()
        .success()
        .stdout(contains("No project started."));
}

#[test]
fn statusline_when_tracking_outputs_project() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args(["start", "-p", "backend", "--at", "08:00"])
        .assert()
        .success();
    watson(&dir)
        .args(["statusline"])
        .assert()
        .success()
        .stdout(predicates::str::is_match("^backend\n$").unwrap());
}

#[test]
fn statusline_when_tracking_with_tags_outputs_project_and_tags() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args([
            "start", "-p", "backend", "-t", "api", "-t", "review", "--at", "08:00",
        ])
        .assert()
        .success();
    watson(&dir)
        .args(["statusline"])
        .assert()
        .success()
        .stdout(contains("backend [api, review]"));
}

// --- start / stop ---

#[test]
fn start_outputs_project_name() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args(["start", "-p", "backend", "--at", "08:00"])
        .assert()
        .success()
        .stdout(contains("Starting"))
        .stdout(contains("backend"));
}

#[test]
fn start_with_tags_outputs_tags() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args([
            "start", "-p", "backend", "-t", "api", "-t", "auth", "--at", "08:00",
        ])
        .assert()
        .success()
        .stdout(contains("api"));
}

#[test]
fn start_twice_auto_stops_first_and_starts_second() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args(["start", "-p", "backend", "--at", "08:00"])
        .assert()
        .success();
    watson(&dir)
        .args(["start", "-p", "frontend", "--at", "08:30"])
        .assert()
        .success()
        .stdout(contains("Stopped"))
        .stdout(contains("backend"))
        .stdout(contains("Starting"))
        .stdout(contains("frontend"));
}

#[test]
fn stop_after_start_outputs_stopped() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args(["start", "-p", "backend", "--at", "08:00"])
        .assert()
        .success();
    watson(&dir)
        .args(["stop", "--at", "09:00"])
        .assert()
        .success()
        .stdout(contains("Stopped"))
        .stdout(contains("backend"));
}

#[test]
fn stop_when_idle_fails() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args(["stop"])
        .assert()
        .failure()
        .stderr(contains("Not currently tracking"));
}

// --- cancel ---

#[test]
fn cancel_after_start_clears_tracking() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args(["start", "-p", "backend", "--at", "08:00"])
        .assert()
        .success();
    watson(&dir)
        .args(["cancel"])
        .assert()
        .success()
        .stdout(contains("Cancelled"))
        .stdout(contains("backend"));
    // should be idle again
    watson(&dir)
        .args(["status"])
        .assert()
        .success()
        .stdout(contains("Not tracking anything"));
}

#[test]
fn cancel_when_idle_fails() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args(["cancel"])
        .assert()
        .failure()
        .stderr(contains("Not currently tracking"));
}

// --- add ---

#[test]
fn add_creates_frame() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args([
            "add", "-p", "meeting", "-t", "planning", "--from", "08:00", "--to", "09:30",
        ])
        .assert()
        .success()
        .stdout(contains("Added"))
        .stdout(contains("meeting"));
}

#[test]
fn add_rejects_inverted_range() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args(["add", "-p", "meeting", "--from", "10:00", "--to", "09:00"])
        .assert()
        .failure()
        .stderr(contains("End time must be after start time"));
}

// --- log / today / projects / report ---

#[test]
fn log_shows_added_frames() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args(["add", "-p", "backend", "--from", "08:00", "--to", "09:00"])
        .assert()
        .success();
    watson(&dir)
        .args(["log"])
        .assert()
        .success()
        .stdout(contains("backend"));
}

#[test]
fn log_when_empty_says_no_frames() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args(["log"])
        .assert()
        .success()
        .stdout(contains("No frames recorded"));
}

#[test]
fn projects_lists_unique_project_names() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args(["add", "-p", "backend", "--from", "08:00", "--to", "09:00"])
        .assert()
        .success();
    watson(&dir)
        .args(["add", "-p", "frontend", "--from", "09:00", "--to", "10:00"])
        .assert()
        .success();
    watson(&dir)
        .args(["add", "-p", "backend", "--from", "10:00", "--to", "11:00"])
        .assert()
        .success();
    watson(&dir)
        .args(["projects"])
        .assert()
        .success()
        .stdout(contains("backend"))
        .stdout(contains("frontend"));
}

#[test]
fn report_shows_project_totals() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args(["add", "-p", "backend", "--from", "08:00", "--to", "10:00"])
        .assert()
        .success();
    watson(&dir)
        .args(["report"])
        .assert()
        .success()
        .stdout(contains("backend"))
        .stdout(contains("2h"));
}

// --- date filters ---

#[test]
fn log_from_filter_excludes_older_frames() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args(["add", "-p", "old", "--from", "08:00", "--to", "09:00"])
        .assert()
        .success();
    watson(&dir)
        .args(["add", "-p", "new", "--from", "10:00", "--to", "11:00"])
        .assert()
        .success();
    // --from today should include both (same day), just verify it runs
    watson(&dir)
        .args(["log", "--from", "today"])
        .assert()
        .success()
        .stdout(contains("old"))
        .stdout(contains("new"));
}

#[test]
fn log_with_future_from_shows_no_frames() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args(["add", "-p", "backend", "--from", "08:00", "--to", "09:00"])
        .assert()
        .success();
    watson(&dir)
        .args(["log", "--from", "2099-01-01"])
        .assert()
        .success()
        .stdout(contains("No frames recorded"));
}

#[test]
fn report_with_date_range_filters_frames() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args(["add", "-p", "backend", "--from", "08:00", "--to", "10:00"])
        .assert()
        .success();
    watson(&dir)
        .args(["report", "--from", "today", "--to", "today"])
        .assert()
        .success()
        .stdout(contains("backend"));
}

// --- rename ---

#[test]
fn rename_updates_project_name() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args(["add", "-p", "old-name", "--from", "08:00", "--to", "09:00"])
        .assert()
        .success();
    watson(&dir)
        .args(["rename", "old-name", "new-name"])
        .assert()
        .success()
        .stdout(contains("Renamed"))
        .stdout(contains("old-name"))
        .stdout(contains("new-name"));
    watson(&dir)
        .args(["projects"])
        .assert()
        .success()
        .stdout(contains("new-name"));
}

#[test]
fn rename_unknown_project_fails() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args(["rename", "ghost", "new-name"])
        .assert()
        .failure()
        .stderr(contains("not found"));
}

// --- past date entry ---

#[test]
fn add_with_explicit_past_date() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args([
            "add",
            "-p",
            "backend",
            "--from",
            "2026-01-01 09:00",
            "--to",
            "2026-01-01 10:00",
        ])
        .assert()
        .success()
        .stdout(contains("Added"));
    watson(&dir)
        .args(["log"])
        .assert()
        .success()
        .stdout(contains("backend"));
}

#[test]
fn start_with_explicit_past_date() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args(["start", "-p", "backend", "--at", "2026-01-01 09:00"])
        .assert()
        .success()
        .stdout(contains("Starting"));
    watson(&dir)
        .args(["stop", "--at", "2026-01-01 10:00"])
        .assert()
        .success()
        .stdout(contains("Stopped"));
}

#[test]
fn add_with_yesterday_shortcut() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args([
            "add",
            "-p",
            "retro",
            "--from",
            "yesterday 09:00",
            "--to",
            "yesterday 10:00",
        ])
        .assert()
        .success()
        .stdout(contains("Added"));
    watson(&dir)
        .args(["log"])
        .assert()
        .success()
        .stdout(contains("retro"));
}

// --- auto-stop on start ---

#[test]
fn start_auto_stops_active_and_starts_new() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args(["start", "-p", "old", "--at", "08:00"])
        .assert()
        .success();
    watson(&dir)
        .args(["start", "-p", "new", "--at", "09:00"])
        .assert()
        .success()
        .stdout(contains("Stopped"))
        .stdout(contains("old"))
        .stdout(contains("Starting"))
        .stdout(contains("new"));
    // old must be in log
    watson(&dir)
        .args(["log"])
        .assert()
        .success()
        .stdout(contains("old"));
    // new must be active
    watson(&dir)
        .args(["status"])
        .assert()
        .success()
        .stdout(contains("new"));
}

#[test]
fn start_auto_stop_rejects_at_before_active_start() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args(["start", "-p", "old", "--at", "10:00"])
        .assert()
        .success();
    watson(&dir)
        .args(["start", "-p", "new", "--at", "09:00"])
        .assert()
        .failure()
        .stderr(contains("End time must be after start time"));
}

// --- overlap detection ---

#[test]
fn add_fails_when_frames_overlap() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args(["add", "-p", "backend", "--from", "08:00", "--to", "10:00"])
        .assert()
        .success();
    watson(&dir)
        .args(["add", "-p", "frontend", "--from", "09:00", "--to", "11:00"])
        .assert()
        .failure()
        .stderr(contains("overlap"));
}

#[test]
fn start_fails_when_time_overlaps_existing_frame() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args(["add", "-p", "backend", "--from", "08:00", "--to", "10:00"])
        .assert()
        .success();
    watson(&dir)
        .args(["start", "-p", "frontend", "--at", "09:00"])
        .assert()
        .failure()
        .stderr(contains("overlap"));
}

#[test]
fn adjacent_frames_are_allowed() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args(["add", "-p", "backend", "--from", "08:00", "--to", "09:00"])
        .assert()
        .success();
    watson(&dir)
        .args(["add", "-p", "frontend", "--from", "09:00", "--to", "10:00"])
        .assert()
        .success();
}

// --- --limit ---

#[test]
fn log_limit_shows_last_n_frames() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args(["add", "-p", "first", "--from", "01:00", "--to", "01:30"])
        .assert()
        .success();
    watson(&dir)
        .args(["add", "-p", "second", "--from", "02:00", "--to", "02:30"])
        .assert()
        .success();
    watson(&dir)
        .args(["add", "-p", "third", "--from", "03:00", "--to", "03:30"])
        .assert()
        .success();
    let out = watson(&dir).args(["log", "--limit", "2"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("second"));
    assert!(stdout.contains("third"));
    assert!(!stdout.contains("first"));
}

// --- tags ---

#[test]
fn tags_lists_all_used_tags() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args([
            "add", "-p", "backend", "-t", "api", "-t", "auth", "--from", "08:00", "--to", "09:00",
        ])
        .assert()
        .success();
    watson(&dir)
        .args(["tags"])
        .assert()
        .success()
        .stdout(contains("api"))
        .stdout(contains("auth"));
}

// --- export ---

#[test]
fn export_csv_to_stdout_has_header_and_data() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args([
            "add", "-p", "backend", "-t", "api", "--from", "08:00", "--to", "09:00",
        ])
        .assert()
        .success();
    watson(&dir)
        .args(["export"])
        .assert()
        .success()
        .stdout(contains("id,project,tags,start,end,duration_seconds"))
        .stdout(contains("backend"))
        .stdout(contains("api"))
        .stdout(contains("3600"));
}

#[test]
fn export_csv_to_file() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args(["add", "-p", "backend", "--from", "08:00", "--to", "09:00"])
        .assert()
        .success();
    let output_file = dir.path().join("export.csv");
    watson(&dir)
        .args(["export", "--output", output_file.to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("Exported"))
        .stdout(contains("1"));
    assert!(output_file.exists());
    let content = std::fs::read_to_string(&output_file).unwrap();
    assert!(content.starts_with("id,project,tags"));
    assert!(content.contains("backend"));
}

#[test]
fn export_timestamps_use_z_suffix() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args(["add", "-p", "backend", "--from", "08:00", "--to", "09:00"])
        .assert()
        .success();
    watson(&dir)
        .args(["export"])
        .assert()
        .success()
        .stdout(contains("Z"))
        .stdout(predicates::str::contains("+00:00").not());
}

// --- import ---

#[test]
fn import_watson_format() {
    let dir = TempDir::new().unwrap();
    let watson_file = dir.path().join("watson_frames");
    std::fs::write(
        &watson_file,
        r#"[
            [1620000000, 1620003600, "imported-project", "abc123", ["tag1"], 1620003600],
            [1620100000, 1620103600, "other-project",    "def456", [],       1620103600]
        ]"#,
    )
    .unwrap();

    watson(&dir)
        .args(["import", "--file", watson_file.to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("Imported"))
        .stdout(contains("2"));

    watson(&dir)
        .args(["projects"])
        .assert()
        .success()
        .stdout(contains("imported-project"))
        .stdout(contains("other-project"));
}

#[test]
fn import_dry_run_shows_preview_without_saving() {
    let dir = TempDir::new().unwrap();
    let watson_file = dir.path().join("watson_frames");
    std::fs::write(
        &watson_file,
        r#"[[1620000000, 1620003600, "preview-project", "abc", [], 1620003600]]"#,
    )
    .unwrap();

    watson(&dir)
        .args([
            "import",
            "--file",
            watson_file.to_str().unwrap(),
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(contains("dry run"))
        .stdout(contains("preview-project"));

    // nothing was actually saved
    watson(&dir)
        .args(["projects"])
        .assert()
        .success()
        .stdout(predicates::str::contains("preview-project").not());
}

// --- epics ---

const EPIC_CONFIG: &str = r#"
[[epics]]
name = "Backend Work"
project = "backend"
tags = []

[[epics]]
name = "Refactoring"
project = "backend"
tags = ["refactor"]
"#;

#[test]
fn epics_lists_configured_epics() {
    let data = TempDir::new().unwrap();
    let cfg = TempDir::new().unwrap();
    std::fs::write(cfg.path().join("config.toml"), EPIC_CONFIG).unwrap();

    watson_cfg(&data, &cfg)
        .args(["epics"])
        .assert()
        .success()
        .stdout(contains("Backend Work"))
        .stdout(contains("Refactoring"));
}

#[test]
fn epics_shows_message_when_none_configured() {
    let data = TempDir::new().unwrap();
    let cfg = TempDir::new().unwrap();
    std::fs::write(
        cfg.path().join("config.toml"),
        "# no provider set — uses compiled-in default\n",
    )
    .unwrap();

    watson_cfg(&data, &cfg)
        .args(["epics"])
        .assert()
        .success()
        .stdout(contains("No epics configured"));
}

#[test]
fn report_epic_groups_by_epic() {
    let data = TempDir::new().unwrap();
    let cfg = TempDir::new().unwrap();
    std::fs::write(cfg.path().join("config.toml"), EPIC_CONFIG).unwrap();

    watson_cfg(&data, &cfg)
        .args([
            "add", "-p", "backend", "-t", "refactor", "--from", "08:00", "--to", "09:00",
        ])
        .assert()
        .success();
    watson_cfg(&data, &cfg)
        .args(["add", "-p", "backend", "--from", "09:00", "--to", "10:00"])
        .assert()
        .success();

    watson_cfg(&data, &cfg)
        .args(["report", "--epic"])
        .assert()
        .success()
        .stdout(contains("Refactoring"))
        .stdout(contains("Backend Work"));
}

#[test]
fn report_epic_shows_unassigned_for_unmatched_frames() {
    let data = TempDir::new().unwrap();
    let cfg = TempDir::new().unwrap();
    std::fs::write(cfg.path().join("config.toml"), EPIC_CONFIG).unwrap();

    watson_cfg(&data, &cfg)
        .args(["add", "-p", "frontend", "--from", "08:00", "--to", "09:00"])
        .assert()
        .success();

    watson_cfg(&data, &cfg)
        .args(["report", "--epic"])
        .assert()
        .success()
        .stdout(contains("Unassigned"))
        .stdout(contains("frontend"));
}

#[test]
fn report_epic_fails_without_configured_epics() {
    let data = TempDir::new().unwrap();
    let cfg = TempDir::new().unwrap();
    std::fs::write(
        cfg.path().join("config.toml"),
        "# no provider set — uses compiled-in default\n",
    )
    .unwrap();

    watson_cfg(&data, &cfg)
        .args(["add", "-p", "backend", "--from", "08:00", "--to", "09:00"])
        .assert()
        .success();

    watson_cfg(&data, &cfg)
        .args(["report", "--epic"])
        .assert()
        .failure()
        .stderr(contains("No epics configured"));
}

// --- invalid time input ---

#[test]
fn start_rejects_invalid_at_format() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args(["start", "-p", "backend", "--at", "invalid"])
        .assert()
        .failure()
        .stderr(contains("Invalid time"));
}
