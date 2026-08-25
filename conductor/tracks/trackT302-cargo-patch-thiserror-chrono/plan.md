# T302 Plan — thiserror + chrono patches

**Status:** **Pending**. Spec [spec.md](./spec.md).
**Ledger:** series DOCS `30b7ca9d`. Fold-in DOCS `a7caf3bc`. Implement **CHORE** on go.

## Phase 0

- [ ] Branch `track/T302-cargo-patch-thiserror-chrono`
- [ ] Re-read crates.io thiserror 2.0.x + chrono 0.4.x latest (AC1/AC2 may track a newer same-line patch)
- [ ] Confirm `cargo pkgid thiserror` still lists **both** `thiserror@1.0.69` and `thiserror@2.0.18` (or current 2.x)
- [ ] `cargo tree -i thiserror@2.0.18` ; `cargo tree -i thiserror@1.0.69` ; `cargo tree -i chrono@0.4.44`
- [ ] Do **not** merge Dependabot remotes `#60` / `#62`
- [ ] CHORE TX

## Tasks

- [ ] `cargo update -p thiserror@2.0.18 --precise 2.0.20` (F8 — **not** bare `-p thiserror`)
- [ ] `cargo update -p chrono@0.4.44 --precise 0.4.45`
- [ ] Confirm lock: thiserror **2.0.20**, chrono **0.4.45**, thiserror **1.0.69** still present (AC1/AC2/AC7)
- [ ] Allow F9 extras: `thiserror-impl` → `syn 3.0.3`; `iana-time-zone` windows-core **0.61.2**. Do **not** revert. Abort if rusqlite/tokio/tower-http/clap versions move
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` ; `cargo nextest run --workspace` ; `cargo deny check` ; `cargo audit`
- [ ] CHANGELOG Unreleased chore row
- [ ] PR body: note syn 3 already in lock; windows-core edge expected (OpenCode O2)
- [ ] PR → watch CI → squash (never `git push origin main`)

## DoD

- [ ] Lock pins thiserror 2.0.20 + chrono 0.4.45 (or execute-current same lines)
- [ ] `thiserror 1.0.69` unchanged; F9 transitives only; no rusqlite/tokio/tower-http/clap bump (AC7)
- [ ] Workspace carets still 2.0 / 0.4 (AC3); `git diff -- crates/` empty (AC5)
- [ ] Full gate green (AC4); CHANGELOG (AC6)
