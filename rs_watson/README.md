# rs_watson

Core library for the rs_watson time tracker. Provides the `Watson` struct with all business logic, domain types, configuration, and epic resolution.

## What's in this crate

| Item | Description |
|---|---|
| `Watson<S>` | Main entry point — all tracking operations |
| `Frame` / `ActiveFrame` | Domain types for completed and in-progress sessions |
| `Report` | Aggregated totals by project and tag |
| `config::Config` | Application configuration (loaded from `config.toml`) |
| `config::EpicConfig` | Single epic rule (project + tag filter → name) |
| `resolve_epic` | Match a frame against a list of epic rules |
| `WatsonError<E>` | Typed error enum for all Watson operations |
| `StartResult` | Return type of `start_or_replace` |

## Usage

```toml
[dependencies]
rs_watson = { path = "../rs_watson", features = ["storage-sqlite"] }
rs_watson_storage = { path = "../rs_watson_storage", default-features = false }
```

```rust
use rs_watson::config::Config;
use rs_watson::Watson;
use rs_watson_storage::sqlite::SqliteStorage;
use chrono::Utc;

let config = Config::load()?;
let storage = SqliteStorage::new("watson.db")?;
let watson = Watson::new(storage);

// Start a session
let active = watson.start("backend", vec!["api".into()], Utc::now())?;

// Check status
if let Some(active) = watson.status()? {
    println!("Tracking: {}", active.project);
}

// Stop
let frame = watson.stop(Utc::now())?;
println!("Logged {} for {}", frame.end - frame.start, frame.project);
```

## `Watson<S>` API

```rust
// Tracking
fn start(project, tags, at)          -> Result<ActiveFrame, _>
fn start_or_replace(project, tags, at) -> Result<StartResult, _>
fn stop(at)                          -> Result<Frame, _>
fn cancel()                          -> Result<ActiveFrame, _>
fn status()                          -> Result<Option<ActiveFrame>, _>

// Frames
fn log()                             -> Result<Vec<Frame>, _>
fn add(project, tags, start, end)    -> Result<Frame, _>
fn edit(id, project, tags, start, end) -> Result<Frame, _>
fn remove(id)                        -> Result<Frame, _>
fn import_frames(frames)             -> Result<usize, _>

// Projects & tags
fn projects()                        -> Result<Vec<String>, _>
fn tags()                            -> Result<Vec<String>, _>
fn rename(from, to)                  -> Result<usize, _>
```

All methods validate inputs (e.g. `end > start`, no overlapping frames) before writing to storage.

## Configuration

```rust
use rs_watson::config::Config;

let config = Config::load()?;
// reads $RS_WATSON_CONFIG_DIR/config.toml  or  ~/.config/rs_watson/config.toml
```

`Config` is serialisable with serde, so you can also construct it in code or write it out with `toml::to_string(&config)`.

## Error handling

`WatsonError<E>` is a typed enum — consumers can match on specific variants:

```rust
use rs_watson::WatsonError;

match watson.stop(Utc::now()) {
    Err(WatsonError::NotTracking)         => { /* nothing active */ }
    Err(WatsonError::InvalidTimeRange)    => { /* at <= active.start */ }
    Err(WatsonError::OverlappingFrame(p)) => { /* would overlap project p */ }
    Err(WatsonError::Storage(e))          => { /* backend I/O error */ }
    Ok(frame) => { /* success */ }
    _ => {}
}
```

## Feature flags

| Flag | Description |
|---|---|
| `storage-sqlite` | Enable the SQLite storage backend |
| `storage-json` | Enable the JSON flat-file storage backend |

At least one must be enabled. The flags gate `StorageProvider` enum variants and propagate down to `rs_watson_storage`.
