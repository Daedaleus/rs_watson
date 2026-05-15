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
4. **Tell the user** the branch name so they can open a PR
5. **Ask whether to push** — after completing the task, always ask: "Soll ich den Branch pushen?"

- If I am already on a non-`main` branch that matches the task, I continue on it
- Hotfixes on `main` are only allowed when explicitly instructed by the user

---

# Project Overview

**rs_watson** is a Rust reimplementation of [Watson](https://github.com/jazzband/Watson), a time-tracking CLI tool.

The project is structured as a Cargo workspace with three crates:

| Crate | Type | Purpose |
|---|---|---|
| `rs_watson` | Library | Core time-tracking logic (reimplementation of Watson's business logic) |
| `rs_watson_storage` | Library | Storage engine abstraction — initial backends: JSON and SQLite |
| `rs_watson_cli` | Binary | CLI interface — mirrors the Watson CLI UX |

- `rs_watson` depends on `rs_watson_storage` for persistence
- `rs_watson_cli` depends on `rs_watson` for all logic
- `rs_watson_storage` has no dependency on the other crates — it is a pure storage abstraction

---

# Architecture Principles

- Strict separation of concerns: logic, storage, and CLI are independent crates
- Storage backends are interchangeable via a trait — no storage-specific code in the logic layer
- No business logic in the CLI crate — only argument parsing and output formatting
- Error handling via `thiserror` / `anyhow` — no `unwrap()` in production paths

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

`logic` · `storage` · `cli` · `json` · `sqlite` · `deps` · `workspace` · `ci`

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
