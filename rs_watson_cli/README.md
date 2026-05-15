# rs_watson_cli

The `watson` command-line binary — a full-featured time tracker that stores frames locally in SQLite or JSON.

## Installation

```sh
cargo install --path .                                        # SQLite (default)
cargo install --path . --no-default-features --features storage-json  # JSON only
```

## Commands

### Tracking

```sh
watson start -p <project> [-t <tag>]... [--at <datetime>]
watson stop  [--at <datetime>]
watson cancel
watson status
```

`start` automatically stops any currently active session before starting the new one.

### Viewing

```sh
watson log    [--from <date>] [--to <date>] [--limit N] [--offset N]
watson today  [--epic]
watson report [--from <date>] [--to <date>] [--epic]
```

### Editing

```sh
watson add    -p <project> [-t <tag>]... --from <datetime> --to <datetime>
watson edit                  # interactive selector
watson remove                # interactive selector with confirmation
watson rename <from> <to>    # renames a project across all frames
```

### Meta

```sh
watson projects
watson tags
watson epics
```

### Import / Export

```sh
watson export [--format csv] [--output <file>] [--from <date>] [--to <date>]
watson import [--source watson] [--file <path>] [--dry-run]
```

`import --source watson` reads the original Python Watson `frames` file. Omit `--file` to use the default Watson data path.

### Shell completions

```sh
watson completions bash  >> ~/.bashrc
watson completions zsh   >> ~/.zshrc
watson completions fish  > ~/.config/fish/completions/watson.fish
```

### Setup

```sh
watson init   # create config.toml interactively
```

---

## Date and time formats

**Dates** (`--from`, `--to`):

| Input | Meaning |
|---|---|
| `YYYY-MM-DD` | Exact date |
| `today` | Today |
| `yesterday` | Yesterday |
| `week` | Start of current week (respects `week_start`) |
| `month` | First day of current month |

**Datetimes** (`--at`, `--from` for `add`):

| Input | Meaning |
|---|---|
| `HH:MM` / `HH:MM:SS` | Today at the given time |
| `YYYY-MM-DD HH:MM[:SS]` | Exact date and time |
| `today HH:MM[:SS]` | Today at the given time |
| `yesterday HH:MM[:SS]` | Yesterday at the given time |

All input is interpreted as **local time** and stored as UTC.

---

## Configuration

File location: `~/.config/rs_watson/config.toml`  
Override with: `RS_WATSON_CONFIG_DIR=/path watson ...`

```toml
[storage]
provider = "sqlite"          # "sqlite" (default) or "json"
# data_dir = "/custom/path"  # default: ~/.local/share/rs_watson

[behavior]
allow_future_times = false   # reject datetimes in the future
week_start = "monday"        # "monday" (default) or "sunday"

[log]
default_limit = 0            # 0 = unlimited; N = show last N frames

[[epics]]
name    = "Sprint 12"
project = "backend"
tags    = ["api", "auth"]    # all tags must be present; empty matches any tag

[[epics]]
name    = "Frontend work"
project = "frontend"
tags    = []
```

### Epics

Epics map a project + tag combination to a named grouping shown in `today --epic` and `report --epic`. The most specific match (most tags) wins. Frames that match no epic are shown under **Unassigned**.

---

## Environment variables

| Variable | Description |
|---|---|
| `RS_WATSON_DATA_DIR` | Overrides `storage.data_dir` and the default XDG path |
| `RS_WATSON_CONFIG_DIR` | Overrides the config file directory |

---

## Feature flags

| Flag | Default | Description |
|---|---|---|
| `storage-sqlite` | ✓ | Enable SQLite backend (requires a C compiler) |
| `storage-json` | — | Enable JSON flat-file backend |

At least one backend must be enabled. Both can be enabled simultaneously; `sqlite` takes priority as the default provider.
