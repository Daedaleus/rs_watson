# rs_watson_storage

Storage abstraction for rs_watson. Defines the `Storage` trait and provides two backend implementations: **SQLite** and **JSON**.

## Storage trait

```rust
pub trait Storage {
    type Error: std::error::Error;

    fn load_frames(&self)  -> Result<Vec<FrameRecord>, Self::Error>;
    fn save_frames(&self, frames: &[FrameRecord]) -> Result<(), Self::Error>;

    fn load_active(&self)  -> Result<Option<ActiveFrameRecord>, Self::Error>;
    fn save_active(&self, frame: Option<&ActiveFrameRecord>) -> Result<(), Self::Error>;
}
```

Implementors define an associated `Error` type. The `rs_watson` library is generic over any type implementing this trait — no backend-specific code leaks into the logic layer.

## Record types

```rust
pub struct FrameRecord {
    pub id:      Uuid,
    pub project: String,
    pub tags:    Vec<String>,
    pub start:   DateTime<Utc>,
    pub end:     DateTime<Utc>,
}

pub struct ActiveFrameRecord {
    pub project: String,
    pub tags:    Vec<String>,
    pub start:   DateTime<Utc>,
}
```

## Backends

### SQLite (`feature = "storage-sqlite"`)

```rust
use rs_watson_storage::sqlite::SqliteStorage;

let storage = SqliteStorage::new("watson.db")?;
```

- Single file database, WAL journal mode
- Tags stored in a normalised `frame_tags` table with positional ordering preserved
- Schema managed via [rusqlite_migration](https://crates.io/crates/rusqlite_migration) — migrations run automatically on open
- Requires a C compiler (bundled `libsqlite3` via rusqlite)

### JSON (`feature = "storage-json"`)

```rust
use rs_watson_storage::json::JsonStorage;

let storage = JsonStorage::new("/path/to/data/dir");
```

- Two files: `frames.json` (all completed frames) and `state.json` (active frame)
- Atomic writes via `.tmp` rename — a crash mid-write never corrupts the real file
- No C compiler required — suitable for environments without a C toolchain

## Feature flags

| Flag | Default | Description |
|---|---|---|
| `json` | ✓ | Enable `JsonStorage` |
| `sqlite` | ✓ | Enable `SqliteStorage` |

Both are on by default. In practice, consumer crates (`rs_watson_cli`, `rs_watson_ui`) set `default-features = false` and enable only the backends they need.

## Usage

```toml
[dependencies]
rs_watson_storage = { path = "../rs_watson_storage", default-features = false, features = ["sqlite"] }
```

```rust
use rs_watson_storage::{Storage, FrameRecord};
use rs_watson_storage::sqlite::SqliteStorage;

let s = SqliteStorage::new(":memory:")?;
s.save_frames(&[/* ... */])?;
let frames = s.load_frames()?;
```
