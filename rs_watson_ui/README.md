# rs_watson_ui

Native desktop UI for rs_watson, built with [egui](https://github.com/emilk/egui) / [eframe](https://github.com/emilk/egui/tree/master/crates/eframe). Reads the same `config.toml` and data files as the CLI — both tools can be used side by side.

## Running

```sh
cargo run -p rs_watson_ui
# or, after installing:
rs_watson_ui
```

Override data / config paths with environment variables before launching:

```sh
RS_WATSON_DATA_DIR=/my/data rs_watson_ui
```

## Features

### Toolbar (always visible)
- **Project** field with autocomplete suggestions from existing projects
- **Tags** field (comma-separated)
- **▶ Start** — starts tracking; automatically stops any active session first
- **■ Stop** — stops and saves the current session
- **✕ Cancel** — discards the current session without saving
- Live status bar showing the tracked project and elapsed time (updated every second)

### Log tab
- All recorded frames grouped by day, newest first
- **Date filter** (`From` / `To`) — accepts `YYYY-MM-DD`, `today`, `yesterday`
- Per-row **Edit** button — opens a modal to change project, tags, start and end time
- Per-row **✕** button — shows an inline delete confirmation before removing

### Add tab
- Form for adding retroactive frames (project, tags, start, end)
- Accepts `YYYY-MM-DD HH:MM[:SS]` or `HH:MM[:SS]` (today's date assumed)
- Project field has the same autocomplete as the toolbar

### Report tab
- Aggregated project / tag totals
- **Date filter** — same shortcuts as the Log tab
- **By Epic** toggle — groups frames by configured epics (only shown when `[[epics]]` entries exist in `config.toml`)

## Configuration

Shared with the CLI. See the [CLI README](../rs_watson_cli/README.md#configuration) for the full reference. The UI respects all settings: storage provider, data directory, week start, and epics.

## System requirements

| Platform | Notes |
|---|---|
| Linux | Requires X11 or Wayland at runtime; see below for build deps |
| macOS | No extra dependencies (Metal) |
| Windows | No extra dependencies (DirectX / Vulkan) |

### Linux build dependencies

```sh
sudo apt-get install -y \
  libxkbcommon-dev libwayland-dev libx11-dev \
  libegl1-mesa-dev libfontconfig1-dev
```
