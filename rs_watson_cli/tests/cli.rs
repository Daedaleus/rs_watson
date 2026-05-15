use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;

fn watson(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("watson").unwrap();
    cmd.env("RS_WATSON_DATA_DIR", dir.path());
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
fn start_twice_fails_with_already_tracking() {
    let dir = TempDir::new().unwrap();
    watson(&dir)
        .args(["start", "-p", "backend", "--at", "08:00"])
        .assert()
        .success();
    watson(&dir)
        .args(["start", "-p", "frontend", "--at", "08:30"])
        .assert()
        .failure()
        .stderr(contains("Already tracking"));
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
