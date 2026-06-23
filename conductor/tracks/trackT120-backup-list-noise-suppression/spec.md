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

**AC1:** "Could not read backup metadata: no such table: _aibrains_backup_meta" is demoted from `warn!` to `debug!` ONLY for backups that predate T109 (i.e., backups created before the metadata table was introduced). For newer backups (created after T109), a missing metadata table indicates actual corruption and remains at `warn!`.

**AC2:** "Skipping backup file with unparseable timestamp during list" is demoted from `warn!` to `debug!`. The `(unparseable)` indicator in the table output is sufficient user-facing signal.

**AC3:** Genuinely unexpected metadata read failures (wrong key, corrupted file, I/O error, "database is locked") remain at `warn!`. Only the "no such table" and "unparseable timestamp" cases are demoted.

**AC4:** `backup list` output is clean by default — no WARN lines on stderr for a vault with a mix of old and new backups. With `RUST_LOG=debug`, the debug messages appear.

**AC5:** No regression in `backup list` table output — the data rows and `(no metadata)` / `(unparseable)` indicators are unchanged.

## Design Notes

- **File:** `crates/ai-brains-brain/src/backup.rs` — `list_backups` method (around lines 268-314) and `prune_backups` (similar warnings).
- **Discriminating old vs new backups:** Instead of blindly string-matching "no such table", check whether the backup file is likely pre-T109. Two approaches:
  1. **Timestamp-based:** If the backup's timestamp (from the filename) predates a known cutoff (e.g., before 2026-06-22 when T109 was implemented), treat missing metadata as expected → `debug!`. Otherwise → `warn!`.
  2. **Content-based (preferred):** After the "no such table" error, check if the backup contains ANY of the core tables (`events`, `memory_projection`). If it does but lacks `_aibrains_backup_meta`, it's a pre-T109 backup (expected) → `debug!`. If it lacks even the core tables, it's corrupted → `warn!`.
- Use approach 2 (content-based) as it's robust regardless of filename format or timestamp.
- In `list_backups`:
  - When `read_backup_metadata` returns an error:
    - If the error is "no such table: _aibrains_backup_meta" → check for core tables. If core tables exist → `tracing::debug!`. If core tables also missing → `tracing::warn!` (genuine corruption).
    - For any other error (wrong key, I/O) → `tracing::warn!` (always).
  - Change `tracing::warn!(... "Skipping backup file with unparseable timestamp during list")` to `tracing::debug!`.
- In `prune_backups`:
  - Same pattern: "unparseable timestamp" → `debug!`, other errors → `warn!`.
- The error discrimination should check the `rusqlite::Error` enum or use `err.to_string().contains("no such table")` for the specific table name, then do the core-tables check.

## Files

- `crates/ai-brains-brain/src/backup.rs` — `list_backups` and `prune_backups` warning level changes.

## Tests (TDD)

**Red:** `backup_list__pre_t109_backup__debug_not_warn` — create a backup without the `_aibrains_backup_meta` table (simulating pre-T109), run `backup list`, capture stderr, assert no `WARN` lines for that backup. Set `RUST_LOG=debug`, assert `DEBUG` line appears.

**Red:** `backup_list__corrupted_new_backup__stays_warn` — create a backup file that has neither `_aibrains_backup_meta` NOR core tables (`events`/`memory_projection`), run `backup list`, assert `WARN` line appears for that file (genuine corruption detected).

**Green:** Implement content-based discrimination. Tests pass.

## Verification

- `cargo nextest run -p ai-brains-brain --lib`
- `cargo nextest run -p ai-brains-cli`
- Manual: `ai-brains backup list` on a vault with old+new backups → no WARN lines on stderr.
- Manual: `$env:RUST_LOG = "debug"; ai-brains backup list` → DEBUG lines show for old backups.

## Out of Scope

- Fixing the timestamp parser itself (T123).
- Adding `--quiet` to `backup list` (the noise is already suppressed by demoting to debug).
- Removing the `(no metadata)` / `(unparseable)` indicators from the table output.