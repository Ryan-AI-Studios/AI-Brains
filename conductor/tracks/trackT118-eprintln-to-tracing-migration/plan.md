# Track T118 Plan: eprintln! to tracing! Migration

## Objective
Migrate informational/progress/warning `eprintln!` calls in `ai-brains-cli` to scoped `tracing::info!`/`tracing::warn!`/`tracing::debug!`, leaving `eprintln!` only for errors, interactive prompts, and the global error handler.

## Tasks

- [x] Read all 11 affected source files and `tests/smoke.rs`
- [x] Verify `tracing` and `tracing-subscriber` dependencies in `Cargo.toml`
- [x] Start ChangeGuard transaction for T118
- [x] **Red — TDD tests**
  - [x] Add `backup_create__progress_goes_to_tracing_not_stderr` to `smoke.rs`
  - [x] Add `tracing_filter__external_deps_stay_quiet` to `smoke.rs`
  - [x] Run targeted nextest; confirm new tests fail on raw stderr text
- [x] **Green — migrate eprintln! calls**
  - [x] `main.rs`: replace `tracing_subscriber::fmt::init()` with scoped `EnvFilter`
  - [x] `commands/nightly.rs`: progress/status → `tracing::info!`; non-fatal failures → `tracing::warn!`
  - [x] `commands/daemon.rs`: status/warnings → `tracing::info!`/`tracing::warn!`
  - [x] `commands/project.rs`: errors keep `eprintln!`, info → `tracing::info!`
  - [x] `commands/sync.rs`: warnings → `tracing::warn!`
  - [x] `commands/backup.rs`: progress → `tracing::info!`, WARNING → `tracing::warn!`
  - [x] `commands/forget.rs`: prompts keep `eprint!`, info → `tracing::info!`
  - [x] `commands/symbol_bridge.rs`: info/warnings → `tracing::info!`/`tracing::warn!`
  - [x] `commands/context.rs`: warning → `tracing::warn!`
  - [x] `commands/graph.rs`: progress → `tracing::info!`
  - [x] `commands/recall.rs`: keep `eprintln!` for no-results hint (T102)
- [x] **Verify**
  - [x] `cargo nextest run -p ai-brains-cli`
  - [x] `cargo clippy --workspace --all-targets -- -D warnings`
  - [x] `cargo fmt --check`
- [x] **Review & finalize**
  - [x] Self-review in `review.md`
  - [x] Update `conductor/conductor.md` status to Completed
  - [x] Commit ChangeGuard transaction
  - [x] Append deferred items to `conductor/ISSUES.md` if any

## Notes
- Count: ~62 `eprintln!` calls across 11 files per spec; actual scan found 64 (includes `eprint!` prompts and one extra in recall.rs).
- AC5 (`--quiet` suppresses info/debug) is satisfied by `EnvFilter` + command-specific quiet flags where already implemented; no new quiet plumbing needed.
- Retained `eprintln!` for: `main.rs` runtime failure / ctrl-c / JSON error envelope; `project.rs` ambiguous/no-match exit paths; `backup.rs` prompts; `forget.rs` prompts; `graph.rs` rebuild progress (now info); `recall.rs` no-results hint (T102).
