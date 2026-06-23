# Track T120: `backup list` Noise Suppression

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P2 — 20+ WARN lines per invocation flood stderr for expected conditions.
**Source:** v0.1.1 verification friction #2 + opportunity #2.

---

## Problem Statement

`ai-brains backup list` emits 20+ `WARN` tracing lines for old backups that lack the `_aibrains_backup_meta` table (pre-T109 backups) and for backups with nanosecond-precision timestamps that don't match the parser:

```
WARN ai_brains_brain::backup: Could not read backup metadata: no such table: _aibrains_backup_meta
WARN ai_brains_brain::backup: Skipping backup file with unparseable timestamp during list
```

These are **expected** conditions, not warnings:
- Pre-T109 backups genuinely don't have the metadata table — that's normal, not a problem.
- Old backups with nanosecond timestamps are legitimate files that just use a different format.

Flooding stderr with expected-condition warnings makes it hard to spot real issues and clutters pipeline output.

## Acceptance Criteria

**AC1:** "Could not read backup metadata: no such table: _aibrains_backup_meta" is demoted from `warn!` to `debug!`. This is an expected condition for pre-T109 backups.

**AC2:** "Skipping backup file with unparseable timestamp during list" is demoted from `warn!` to `debug!`. The `(unparseable)` indicator in the table output is sufficient user-facing signal.

**AC3:** Genuinely unexpected metadata read failures (wrong key, corrupted file, I/O error) remain at `warn!`. Only the "no such table" and "unparseable timestamp" cases are demoted.

**AC4:** `backup list` output is clean by default — no WARN lines on stderr for a vault with a mix of old and new backups. With `RUST_LOG=debug`, the debug messages appear.

**AC5:** No regression in `backup list` table output — the data rows and `(no metadata)` / `(unparseable)` indicators are unchanged.

## Design Notes

- **File:** `crates/ai-brains-brain/src/backup.rs` — `list_backups` method (around lines 268-314) and `prune_backups` (similar warnings).
- In `list_backups`:
  - Change `tracing::warn!(... "Could not read backup metadata")` to check the error first. If error message contains "no such table" → `tracing::debug!`. Otherwise → `tracing::warn!`.
  - Change `tracing::warn!(... "Skipping backup file with unparseable timestamp during list")` to `tracing::debug!`.
- In `prune_backups`:
  - Same pattern: "unparseable timestamp" → `debug!`, other errors → `warn!`.
- The error discrimination can use `err.to_string().contains("no such table")` or check the `rusqlite::Error` enum if it exposes the specific error kind.

## Files

- `crates/ai-brains-brain/src/backup.rs` — `list_backups` and `prune_backups` warning level changes.

## Tests (TDD)

**Red:** `backup_list__expected_conditions_are_debug_not_warn` — create a vault with a pre-T109-style backup (no metadata table) and a new backup. Run `backup list`, capture stderr. Assert no `WARN` lines appear. Set `RUST_LOG=debug`, run again, assert `DEBUG` lines appear for the old backup.

**Green:** Demote the expected-condition warnings to debug. Test passes.

## Verification

- `cargo nextest run -p ai-brains-brain --lib`
- `cargo nextest run -p ai-brains-cli`
- Manual: `ai-brains backup list` on a vault with old+new backups → no WARN lines on stderr.
- Manual: `$env:RUST_LOG = "debug"; ai-brains backup list` → DEBUG lines show for old backups.

## Out of Scope

- Fixing the timestamp parser itself (T123).
- Adding `--quiet` to `backup list` (the noise is already suppressed by demoting to debug).
- Removing the `(no metadata)` / `(unparseable)` indicators from the table output.