# T302 Plan — thiserror + chrono patches

**Status:** **Completed**. Spec [spec.md](./spec.md).
**Ledger:** series DOCS `30b7ca9d`. Fold-in DOCS `a7caf3bc`. Implement CHORE `aec6d64e-82e4-4593-ab1c-628f3112d329`.

## Phase 0

- [x] Branch `track/T302-cargo-patch-thiserror-chrono`
- [x] Re-read crates.io thiserror 2.0.x + chrono 0.4.x latest — still **2.0.20** / **0.4.45** (2026-08-25)
- [x] Confirm `cargo pkgid thiserror` ambiguous: `thiserror@1.0.69` + `thiserror@2.0.18` (pre-update)
- [x] `cargo tree -i thiserror@2.0.18` ; `thiserror@1.0.69` ; `chrono@0.4.44`
- [x] Do **not** merge Dependabot remotes `#60` / `#62`
- [x] CHORE TX `aec6d64e-82e4-4593-ab1c-628f3112d329`

## Tasks

- [x] `cargo update -p thiserror@2.0.18 --precise 2.0.20` (F8)
- [x] `cargo update -p chrono@0.4.44 --precise 0.4.45`
- [x] Confirm lock: thiserror **2.0.20**, chrono **0.4.45**, thiserror **1.0.69** still present (AC1/AC2/AC7)
- [x] F9 extras landed: `thiserror-impl` → `syn 3.0.3`; `iana-time-zone` windows-core **0.61.2**. rusqlite/tokio/tower-http/clap unchanged
- [x] `cargo clippy --workspace --all-targets -- -D warnings` ; `cargo nextest run --workspace` ; `cargo deny check` ; `cargo audit` (via `dev-check.ps1` + ledgerful full — exit 0)
- [x] CHANGELOG Unreleased Changed row
- [ ] PR body: note syn 3 already in lock; windows-core edge expected (OpenCode O2)
- [ ] PR → watch CI → squash (never `git push origin main`)

## DoD

- [x] Lock pins thiserror 2.0.20 + chrono 0.4.45
- [x] `thiserror 1.0.69` unchanged; F9 transitives only; no rusqlite/tokio/tower-http/clap bump (AC7)
- [x] Workspace carets still 2.0 / 0.4 (AC3); `git diff -- crates/` empty (AC5)
- [x] Full gate green (AC4); CHANGELOG (AC6)

## Evidence (execute)

```text
cargo update -p thiserror@2.0.18 --precise 2.0.20  → thiserror + thiserror-impl 2.0.20
cargo update -p chrono@0.4.44 --precise 0.4.45      → chrono 0.4.45
cargo pkgid thiserror → ambiguous thiserror@1.0.69 / thiserror@2.0.20
thiserror-impl@2.0.20 → syn 3.0.3
iana-time-zone 0.1.65 → windows-core 0.61.2 (0.62.2 still in lock)
rusqlite 0.39.0 / tokio 1.52.3 / tower-http 0.6.11 / clap 4.6.1 unchanged
git diff -- crates/ → empty
```

