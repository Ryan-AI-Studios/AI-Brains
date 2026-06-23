# Track T115: Sync Query Daemon Fallback — Plan

## Status
In Progress → Verify → Review

## Goal
Remove the `ensure_running` daemon gate from `ai-brains sync query` so local recall + ChangeGuard search proceed regardless of daemon state.

## Checklist

- [x] Read existing `run_query` in `crates/ai-brains-cli/src/commands/sync.rs`.
- [x] Read existing `smoke.rs` patterns and `sync_query_isolation.rs` regression tests.
- [x] Start ChangeGuard transaction (`ledgerful ledger start T115-sync-query-daemon-fallback`).
- [x] Run `ledgerful scan --impact` and note `sync.rs` hotspot.
- [x] Add TDD red tests to `crates/ai-brains-cli/tests/smoke.rs`:
  - `sync_query__daemon_down__returns_local_results`
  - `sync_query__daemon_down__no_spawn_attempt`
- [x] Confirm new tests fail against old code (red phase).
- [x] Remove daemon `ensure_running` gate from `run_query` (sync.rs:395-402).
- [x] Confirm new tests pass (green phase).
- [x] Run `cargo nextest run -p ai-brains-cli` — all 71 pass.
- [x] Run `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` — clean.
- [x] Run `ledgerful verify --scope fast` — fmt/clippy/tests passed for changed files after targeted `cargo fmt`.
- [ ] Update `conductor/conductor.md` status to Complete.
- [ ] Write `review.md` with self-review findings.
- [ ] Finalize ChangeGuard transaction (`ledgerful ledger commit`).

## Affected Files
- `crates/ai-brains-cli/src/commands/sync.rs`
- `crates/ai-brains-cli/tests/smoke.rs`
- `conductor/conductor.md`
- `conductor/tracks/trackT115-sync-query-daemon-fallback/review.md`

## Out of Scope
- `sync push` / `sync pull` daemon behavior unchanged.
- No new CLI flags.
- No warning added (daemon absence is irrelevant to `sync query`).

## Notes
- Existing `sync_query_isolation.rs` tests cover AC2 (daemon up path is unaffected) implicitly because they don't start a daemon and pass under the new code.
- AC4 (ndjson without daemon) covered by existing `sync_query_ndjson_remains_scoped_no_regression`.
- AC6 (no probe latency) covered by new tests completing in ~1.6s with no daemon.
