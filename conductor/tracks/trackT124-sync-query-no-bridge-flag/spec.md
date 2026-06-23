# Track T124: `sync query --no-bridge` Flag

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P3 — consistency; `recall` has `--no-bridge` but `sync query` doesn't.
**Source:** v0.1.1 verification friction #6.

---

## Problem Statement

`ai-brains recall` has a `--no-bridge` flag that skips the ChangeGuard bridge query and uses only local vault search. `ai-brains sync query` does NOT have this flag, so users can't skip the ChangeGuard `ledgerful search` portion of `sync query`:

```
$ ai-brains sync query "backup" --no-bridge
error: unexpected argument '--no-bridge' found
```

This is inconsistent. In CI environments or non-git directories where `ledgerful` is unavailable, `sync query` still attempts the bridge call and emits warnings (unless `--quiet` is also passed). A `--no-bridge` flag would let users explicitly skip it.

## Acceptance Criteria

**AC1:** `sync query --no-bridge` skips the ChangeGuard `ledgerful search` section entirely. Only local vault recall runs.

**AC2:** `sync query --no-bridge --format pretty` shows only the "AI-Brains Recall" section, no "ChangeGuard Ledger Search" section.

**AC3:** `sync query --no-bridge --format ndjson` emits only local recall records, no ChangeGuard records.

**AC4:** `sync query` (without --no-bridge) works as before — no regression.

**AC5:** `--no-bridge` and `--quiet` are independent. `--no-bridge` skips the bridge call; `--quiet` suppresses warnings from a failed bridge call. Both can be used together.

## Design Notes

- **File:** `crates/ai-brains-cli/src/main.rs` — Add `--no-bridge` flag to the `Sync` subcommand (or `SyncQuery` if separate). Check how `recall`'s `--no-bridge` is declared.
- **File:** `crates/ai-brains-cli/src/commands/sync.rs` — `run_query`: when `no_bridge` is true, skip the `ledgerful search` call and the "ChangeGuard Ledger Search" section output. Still run local recall.

## Files

- `crates/ai-brains-cli/src/main.rs` — Add `--no-bridge` to sync query.
- `crates/ai-brains-cli/src/commands/sync.rs` — `run_query`: gate the bridge section on `no_bridge`.

## Tests (TDD)

**Red:** `sync_query__no_bridge__skips_changeguard_section` — run `sync query "test" --no-bridge --format pretty`, assert output contains "AI-Brains Recall" but NOT "ChangeGuard Ledger Search".

**Red:** `sync_query__no_bridge_ndjson__only_local_records` — run `sync query "test" --no-bridge --format ndjson`, assert all records are local (no ChangeGuard-sourced records).

**Green:** Implement the flag. Tests pass.

## Verification

- `cargo nextest run -p ai-brains-cli`
- Manual: `ai-brains sync query "backup" --no-bridge` → only local recall section.

## Out of Scope

- Adding `--no-bridge` to `sync pull` or `sync push` (those have different semantics).
- Removing the `--quiet` flag (it serves a different purpose — suppressing warnings from a failed bridge, not skipping the bridge entirely).