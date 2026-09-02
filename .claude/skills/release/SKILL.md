---
name: release
description: Cut a new rs_watson release — bump all five workspace crates and their inter-crate dependencies to a new version, verify, land it via PR, then tag and push so the GitHub release workflow builds the binaries. Use when asked to release, cut a version, or bump the workspace version.
---

# Release rs_watson

Bumping a version touches **11 version strings across 5 `Cargo.toml` files plus `Cargo.lock`**.
Missing one produces a workspace that still builds but ships inconsistent metadata, so do it
mechanically and verify with the grep in step 3 — never by hand-editing files one at a time.

## 0. Determine old and new version

```sh
OLD=$(grep -m1 '^version' rs_watson/Cargo.toml | cut -d'"' -f2)
```

Ask the user for `NEW` if they did not say it. Semver against the changes since the last tag:
breaking CLI/storage change → minor while pre-1.0, otherwise patch for fixes, minor for features.

## 1. Branch

```sh
git checkout main && git pull --ff-only
git checkout -b build/bump-v$NEW
```

## 2. Bump every crate and every inter-crate dependency

Both patterns matter: the `[package] version` line, and the `rs_watson*` path dependencies that
pin the same version. Third-party dependency versions must NOT be touched — the anchors below
(`^version` and `rs_watson… = { version =`) are what keeps them out.

```sh
sed -i -e "s/^version = \"$OLD\"/version = \"$NEW\"/" \
       -e "s/\(rs_watson[a-z_]*\) = { version = \"$OLD\"/\1 = { version = \"$NEW\"/" \
       rs_watson/Cargo.toml rs_watson_storage/Cargo.toml rs_watson_export/Cargo.toml \
       rs_watson_cli/Cargo.toml rs_watson_ui/Cargo.toml
cargo check --workspace   # regenerates Cargo.lock
```

## 3. Verify nothing was missed

```sh
grep -rn "\"$OLD\"" */Cargo.toml   # must print NOTHING
grep -rn "^version" */Cargo.toml   # must show $NEW five times
```

If the first grep prints anything, a version string was missed — fix it before continuing.

## 4. Full check suite

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## 5. Commit, PR, merge

```sh
git commit -am "build(workspace): 🔧 bump all crates to v$NEW"
git push -u origin build/bump-v$NEW
gh pr create --base main --title "build(workspace): 🔧 bump all crates to v$NEW"
```

Wait for CI: `gh pr checks <n> --watch`. Merge only when green (squash, matching repo history).

## 6. Tag — this is what actually publishes

`.github/workflows/release.yml` triggers on tags matching `v*.*.*`. Nothing is released until
the tag is pushed, and the tag must point at the merge commit on `main`, not at the branch.

```sh
git checkout main && git pull --ff-only
git tag v$NEW && git push origin v$NEW
```

Confirm the build: `gh run list --limit 3` — the `Release` workflow builds four targets
(Linux x86_64, macOS aarch64, macOS x86_64, Windows x86_64) and attaches the archives to the
GitHub Release. It takes ~8 minutes; report the result rather than assuming success.
