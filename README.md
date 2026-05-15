# rs_watson

A Rust reimplementation of [Watson](https://github.com/jazzband/Watson) — a command-line time tracking tool. Ships as both a **CLI** and a native **desktop UI**, sharing the same data files.

## Crates

| Crate | Type | Description |
|---|---|---|
| [`rs_watson`](rs_watson/) | Library | Core tracking logic, domain types, configuration |
| [`rs_watson_storage`](rs_watson_storage/) | Library | Storage abstraction — JSON and SQLite backends |
| [`rs_watson_export`](rs_watson_export/) | Library | Export functionality (CSV) |
| [`rs_watson_cli`](rs_watson_cli/) | Binary `watson` | Full-featured CLI, mirrors the Watson UX |
| [`rs_watson_ui`](rs_watson_ui/) | Binary `rs_watson_ui` | Native desktop UI (egui/eframe) |

CLI and UI read the same `config.toml` and data files — you can use both interchangeably.

## Installation

### From source

```sh
# CLI only (SQLite backend, default)
cargo install --path rs_watson_cli

# Desktop UI
cargo install --path rs_watson_ui

# CLI with JSON backend instead of SQLite (no C compiler required)
cargo install --path rs_watson_cli --no-default-features --features storage-json
```

### Pre-built binaries

Download the latest release archive for your platform from the [Releases](../../releases) page. Each archive contains both `watson` (CLI) and `rs_watson_ui` (desktop app).

| Platform | Archive |
|---|---|
| Linux x86_64 | `rs-watson-vX.Y.Z-linux-x86_64.tar.gz` |
| macOS ARM64 | `rs-watson-vX.Y.Z-macos-aarch64.tar.gz` |
| macOS x86_64 | `rs-watson-vX.Y.Z-macos-x86_64.tar.gz` |
| Windows x86_64 | `rs-watson-vX.Y.Z-windows-x86_64.zip` |

## Quick start

```sh
# Create the config file interactively
watson init

# Start tracking
watson start -p backend -t api

# Check status
watson status

# Stop and save
watson stop

# View today's log
watson today

# View a report for this week
watson report --from week
```

See the [CLI README](rs_watson_cli/README.md) for the full command reference.

## Configuration

Config is stored in `~/.config/rs_watson/config.toml` (XDG on Linux/macOS, `%APPDATA%` on Windows). Run `watson init` to create it interactively, or write it by hand:

```toml
[storage]
provider = "sqlite"          # "sqlite" (default) or "json"
# data_dir = "/custom/path"  # override default XDG data directory

[behavior]
allow_future_times = false   # reject start/stop times in the future
week_start = "monday"        # "monday" (default) or "sunday"

[log]
default_limit = 0            # 0 = show all; N = show last N frames

[[epics]]
name    = "Backend Sprint"
project = "backend"
tags    = ["api"]            # all listed tags must be present; empty = match all
```

See the [full configuration reference](rs_watson_cli/README.md#configuration) in the CLI README.

## Environment variables

| Variable | Description |
|---|---|
| `RS_WATSON_DATA_DIR` | Override the data directory (frames / database) |
| `RS_WATSON_CONFIG_DIR` | Override the config directory |

## Storage backends

| Backend | Feature flag | Notes |
|---|---|---|
| SQLite | `storage-sqlite` (default) | Single `watson.db` file, WAL mode |
| JSON | `storage-json` | Flat `frames.json` + `state.json`, no C compiler required |

## Development

```sh
# Run all tests
cargo test

# Lint
cargo fmt --check
cargo clippy -- -D warnings

# Run the CLI
cargo run -p rs_watson_cli -- start -p myproject

# Run the UI
cargo run -p rs_watson_ui
```

## License

MIT
