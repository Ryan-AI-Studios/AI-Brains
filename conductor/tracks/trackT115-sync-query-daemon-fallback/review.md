# Track T115: Sync Query Daemon Fallback — Review Log

## Self-Review

### Findings

| id | severity | description | file | status |
|----|----------|-------------|------|--------|
| 1 | info | Removed 8 lines including the `DaemonClient` probe/spawn gate. No other logic changed in `run_query`. | `crates/ai-brains-cli/src/commands/sync.rs` | verified_fixed |
| 2 | info | New smoke tests follow naming `feature__condition__result`, no `test_` prefix, use `tempfile::tempdir()`, and avoid `unwrap()` via `assert_cmd` + explicit asserts. | `crates/ai-brains-cli/tests/smoke.rs` | verified_fixed |
| 3 | info | `run_query` still uses `unwrap_or_else(|_| ...)` for `ProjectId` parsing fallback; this is existing code, not new in this track, and is a defensive fallback rather than an unhandled panic. Left unchanged per scope rule. | `crates/ai-brains-cli/src/commands/sync.rs` | out_of_scope |
| 4 | low | `daemon_client` module remains in `main.rs` and is used by other commands (`backup`, `daemon`, `nightly`). No dead code introduced. | `crates/ai-brains-cli/src/main.rs` | verified_fixed |

### Severity Summary
- Critical: 0
- High: 0
- Medium: 0
- Low/Info: 4 (all acceptable or out_of_scope)

### AC Coverage
- AC1: Covered by new test `sync_query__daemon_down__returns_local_results` + manual run.
- AC2: Covered by existing `sync_query_isolation.rs` tests continuing to pass.
- AC3: `--quiet` path unchanged; no new stderr noise added.
- AC4: Covered by existing `sync_query_ndjson_remains_scoped_no_regression`.
- AC5: All 71 `ai-brains-cli` tests pass.
- AC6: New tests complete in ~1.6s with no daemon; no probe timeout observed.

### Manual Verification
Command run from repo root with no daemon:
```powershell
ai-brains sync query "backup" --format pretty
```
Result: local recall section printed immediately, no daemon error, no perceptible delay.

## Reviewer Sign-off
Self-reviewed by implementer. Clean; no blockers.
