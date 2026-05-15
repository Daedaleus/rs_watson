# General

- Challenge me critically when needed before doing anything
- Use best practices and current versions for each language/ecosystem
- No `Co-Authored-By` lines in commit messages
- Use `cargo` for all project management tasks (creating crates, adding dependencies, etc.)

## Workflow — what I do for every task

1. **Create a branch** before touching any code — never commit directly to `main`:
   ```sh
   git checkout -b feat/some-feature
   ```
2. **Make changes** and commit to that branch using the commit convention below
3. **Run `cargo fmt`** before every commit — mandatory, no exceptions
4. **Merge to `main`** once the task is complete and all checks pass:
   ```sh
   git checkout main && git merge <branch> --ff-only && git branch -d <branch>
   ```
5. **Ask whether to push** — if a remote is configured, ask: "Soll ich den Branch pushen?" before merging

- If I am already on a non-`main` branch that matches the task, I continue on it
- Hotfixes on `main` are only allowed when explicitly instructed by the user

---

# Project Overview

**rs_watson** is a Rust reimplementation of [Watson](https://github.com/jazzband/Watson), a time-tracking CLI tool.

The project is structured as a Cargo workspace with five crates:

| Crate | Type | Purpose |
|---|---|---|
| `rs_watson` | Library | Core logic, domain types, Config, EpicConfig, resolve_epic |
| `rs_watson_storage` | Library | Storage trait + JSON and SQLite backends |
| `rs_watson_export` | Library | Export trait + CSV exporter |
| `rs_watson_cli` | Binary `watson` | CLI interface — mirrors the Watson CLI UX |
| `rs_watson_ui` | Binary `rs_watson_ui` | Native desktop UI (egui/eframe) |

**Dependency graph:**
```
rs_watson_storage  (no deps on other workspace crates)
       ↓
rs_watson          (logic + Config + EpicConfig + resolve_epic)
       ↓              ↓
rs_watson_cli    rs_watson_ui    (both depend on rs_watson; CLI also on rs_watson_export)
```

- `rs_watson_storage` is a pure storage abstraction — no business logic
- `rs_watson` owns all domain logic **and** application configuration (`Config`, `EpicConfig`, `WeekStart`, `StorageProvider`)
- Neither `rs_watson_cli` nor `rs_watson_ui` contain business logic — only I/O, rendering, and argument parsing
- Storage feature flags (`storage-json`, `storage-sqlite`) are defined in `rs_watson` and forwarded by CLI/UI

---

# Architecture Principles

- Strict separation of concerns: logic, storage, CLI, and UI are independent crates
- Storage backends are interchangeable via a trait — no storage-specific code in the logic layer
- No business logic in CLI or UI — only argument parsing, output formatting, and rendering
- Configuration (`Config` and all sub-types) lives in `rs_watson`, not in CLI or UI, so both binaries share the same config format and loading logic without duplication
- Error handling via `thiserror` for library crates, `anyhow` for binaries — no `unwrap()` in production paths
- The UI (`rs_watson_ui`) uses egui/eframe and requires system libraries on Linux (libxkbcommon, libwayland, libx11, libegl, libfontconfig)

---

# Code Standards

## Rust

- `rustfmt` enforced — `cargo fmt` before every commit
- `cargo clippy -- -D warnings` must pass with zero warnings
- `cargo test` must pass
- No `unwrap()` or `expect()` in production code paths
- Use `thiserror` for library errors, `anyhow` for binary errors

---

# Branching Strategy

Branch names follow the same type vocabulary as commits:

| Type | Branch pattern | Example |
|---|---|---|
| New feature | `feat/<topic>` | `feat/frame-overlap-detection` |
| Bug fix | `fix/<topic>` | `fix/json-serialize-tags` |
| Refactor | `refactor/<topic>` | `refactor/storage-trait` |
| Documentation | `docs/<topic>` | `docs/cli-usage` |
| Build / infra | `build/<topic>` | `build/workspace-setup` |
| CI/CD | `ci/<topic>` | `ci/add-clippy-check` |
| Chore | `chore/<topic>` | `chore/update-deps` |

- Use lowercase and hyphens — no underscores, no camelCase
- Keep topics short and descriptive (2–4 words max)
- One logical change per branch

---

# Commit Convention

This project follows [Conventional Commits](https://www.conventionalcommits.org/).

## Format

```
<type>(<scope>): <emoji> <short description>

[optional body — bullet points for details]
```

- **type** — see table below
- **scope** — affected area (see scope examples)
- **emoji** — one emoji directly after the colon, before the description
- **description** — imperative mood, lowercase, no period, English

## Types

| Type | Emoji | When to use |
|---|---|---|
| `feat` | ✨ | New user-facing feature |
| `fix` | 🐛 | Bug fix |
| `refactor` | ♻️ | Code improvement without behaviour change |
| `style` | 🎨 | Formatting only |
| `perf` | ⚡ | Performance improvement |
| `test` | 🧪 | Adding or updating tests |
| `docs` | 📚 | Documentation only |
| `build` | 🔧 | Build system, dependencies, workspace config |
| `ci` | ⚙️ | CI/CD pipeline changes |
| `chore` | 🔨 | Tooling, config, maintenance (no production code) |
| `revert` | ⏪ | Reverts a previous commit |

## Scope Examples

`logic` · `storage` · `cli` · `ui` · `export` · `config` · `json` · `sqlite` · `deps` · `workspace` · `ci`

## Examples

```
feat(logic): ✨ add frame overlap detection
fix(storage): 🐛 fix json deserialization for empty tag list
refactor(storage): ♻️ extract storage trait into separate module
build(workspace): 🔧 add rs_watson_cli binary crate
test(logic): 🧪 add tests for report aggregation
```

## Rules

- One logical change per commit
- Clear technical description — no vague messages ("fix bug", "update code", "stuff")
- Imperative mood ("add", "fix", "remove" — not "added", "fixed", "removed")
- English as standard
- Body optional but encouraged for non-obvious changes

---

# CI / Release

## CI (`.github/workflows/ci.yml`)

Three jobs run on every push to `main` and every PR:

| Job | Runs on | What it checks |
|---|---|---|
| `lint` | ubuntu-latest | `cargo fmt --check`, `cargo clippy -- -D warnings` (full workspace) |
| `test` | ubuntu / macos / windows | `cargo build`, `cargo test` (full workspace) |
| `test-features` | ubuntu-latest | `rs_watson_cli` built/tested with sqlite-only, json-only, and both backends |

Linux runners install egui/eframe system dependencies before building.

## Release (`.github/workflows/release.yml`)

Triggered by a tag matching `v*.*.*`:

```sh
git tag v1.2.3 && git push origin v1.2.3
```

Builds `watson` (CLI) + `rs_watson_ui` in release mode on four targets and attaches them to a GitHub Release:

| Target | Archive |
|---|---|
| Linux x86_64 | `rs-watson-vX.Y.Z-linux-x86_64.tar.gz` |
| macOS ARM64 | `rs-watson-vX.Y.Z-macos-aarch64.tar.gz` |
| macOS x86_64 | `rs-watson-vX.Y.Z-macos-x86_64.tar.gz` |
| Windows x86_64 | `rs-watson-vX.Y.Z-windows-x86_64.zip` |

---

# Pull Request Guidelines

## Title

Follows the same format as a commit message:
```
feat(logic): ✨ add Watson-compatible report output
```

## Body

```markdown
## Summary
- What changed and why (2–5 bullet points)

## Test plan
- [ ] `cargo test` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo fmt` applied
- [ ] Manually tested: <what you ran/tested>

## Related
Closes #<issue> (if applicable)
```

## Rules

- PRs target `main` via a feature/fix branch — no direct pushes to `main`
- One logical feature or fix per PR — keep scope small
- Self-review before requesting review — read your own diff
- CI must be green before merge
