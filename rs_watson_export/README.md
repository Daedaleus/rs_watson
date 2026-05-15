# rs_watson_export

Export library for rs_watson. Defines the `Exporter` trait and provides a CSV implementation.

## Exporter trait

```rust
pub trait Exporter {
    type Error: std::error::Error;

    fn export<W: Write>(&self, frames: &[Frame], writer: W) -> Result<(), Self::Error>;
}
```

Writers are generic over `std::io::Write` — export to a file, stdout, or any byte sink.

## CSV exporter

```rust
use rs_watson_export::csv::CsvExporter;
use rs_watson_export::Exporter;

// To stdout
CsvExporter.export(&frames, std::io::stdout())?;

// To a file
let file = std::fs::File::create("export.csv")?;
CsvExporter.export(&frames, file)?;
```

### Output format

```
id,project,tags,start,end,duration_seconds
3f2a...,backend,api|auth,2026-01-15T09:00:00Z,2026-01-15T10:30:00Z,5400
```

| Column | Format |
|---|---|
| `id` | UUID v4 |
| `project` | Project name |
| `tags` | `\|`-separated list, empty string if none |
| `start` / `end` | RFC 3339 with `Z` suffix (UTC) |
| `duration_seconds` | Integer seconds |

## Usage

```toml
[dependencies]
rs_watson_export = { path = "../rs_watson_export" }
```
