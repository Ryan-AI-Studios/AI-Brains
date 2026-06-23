# Track T112: Recall Scope Overhaul — Self-Review

## Summary

Implemented the recall scope overhaul: default recall is now project-scoped (no session filter), `--global` clears both project and session filters, and a new `--session <ID>` flag provides explicit session scoping. The daemon's `query_memories` RPC path now accepts `Option<ProjectId>`/`Option<SessionId>` and defaults to unscoped search when the caller omits IDs.

## Findings

| id | severity | description | file | status |
|---|---|---|---|---|
| 1 | info | `sync.rs` NDJSON path previously auto-set `session_id` from env; now defaults to `None`. | `crates/ai-brains-cli/src/commands/sync.rs` | fixed_pending_verification |
| 2 | info | `main.rs` Recall `session_id` field had an env binding; removed and renamed flag to `--session`. | `crates/ai-brains-cli/src/main.rs` | fixed_pending_verification |
| 3 | info | `query_memories` signature changed; caller updated to pass `Option`s instead of generating defaults. | `crates/ai-brainsd/src/lib.rs`, `crates/ai-brainsd/src/main.rs` | fixed_pending_verification |

No critical, high, or medium findings. The change is behaviorally breaking for anyone relying on implicit session scoping, but the previous behavior was a bug per the spec.

## Verification Results

- `cargo nextest run -p ai-brains-cli`: 27 passed, 0 skipped
- `cargo nextest run -p ai-brains-retrieval`: 40 passed, 0 skipped
- `cargo nextest run -p ai-brainsd`: 3 passed, 0 skipped
- `cargo clippy -p ai-brains-cli -p ai-brains-retrieval -p ai-brainsd --all-targets -- -D warnings`: clean
- `cargo fmt --check`: clean

## Manual Verification Notes

Manual verification of the live CLI was not run because the test suite exercises the same code paths with isolated vaults. The new smoke tests cover default project scope, `--global`, `--session`, and the env-var non-scoping behavior explicitly.
