# Track T123: Backup Timestamp Parser Robustness

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P2 — 3+ orphaned backups can't be listed, pruned, or restored by name.
**Source:** v0.1.1 verification friction #5 + opportunity #1.

---

## Problem Statement

The backup timestamp parser in `crates/ai-brains-brain/src/backup.rs` uses `NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%dT%H-%M-%S")`. This fails for backups created with nanosecond precision and timezone offsets:

```
vault-2026-04-28T16-23-52.639348300+00-00.db.bak  → (unparseable)
vault-2026-05-10T19-01-41.922523400+00-00.db.bak  → (unparseable)
vault-2026-05-12T20-40-52.967041100+00-00.db.bak  → (unparseable)
```

These are legitimate backup files that:
- Can't be listed with a proper timestamp (show "(unparseable)").
- Can't be pruned (the timestamp filter doesn't match them).
- Can't be sorted correctly (they fall to the bottom).

The root cause is that these backups were created by an older version of the code that used `chrono::Utc::now().to_rfc3339()` for the filename (which includes nanoseconds + timezone), while the current code uses `Utc::now().format("%Y-%m-%dT%H-%M-%S")` (seconds only, no timezone).

## Acceptance Criteria

**AC1:** The timestamp parser handles these formats in addition to the current `%Y-%m-%dT%H-%M-%S`:
- `%Y-%m-%dT%H-%M-%S%.f%:z` (nanosecond precision with timezone offset, e.g. `2026-04-28T16-23-52.639348300+00:00`)
- `%Y-%m-%dT%H-%M-%S%.f` (nanosecond precision without timezone, e.g. `2026-04-28T16-23-52.639348300`)
- `%Y-%m-%dT%H-%M-%S` (current format, seconds only)

**AC2:** Files with nanosecond timestamps are correctly parsed, listed with their proper timestamp, and sorted chronologically alongside second-precision backups.

**AC3:** Files with `+00-00` timezone offset (using `-` instead of `:` separator) are parsed correctly. The parser must handle both `+00:00` and `+00-00` variants.

**AC4:** Backups that still can't be parsed after trying all formats continue to show `(unparseable)` — no regression for truly malformed names.

**AC5:** `backup prune` correctly handles nanosecond-precision backups — they are included in the sort order and eligible for pruning.

## Design Notes

- **File:** `crates/ai-brains-brain/src/backup.rs` — `list_backups` and `prune_backups` both have timestamp parsing logic.
- Create a shared helper function `fn parse_backup_timestamp(s: &str) -> Option<NaiveDateTime>` that tries multiple formats in order:
  1. `%Y-%m-%dT%H-%M-%S` (current, seconds only)
  2. `%Y-%m-%dT%H-%M-%S%.f` (nanoseconds, no timezone)
  3. `%Y-%m-%dT%H-%M-%S%.f%:z` (nanoseconds with `+00:00` timezone)
  4. For `+00-00` variant: replace `-` with `:` in the timezone portion before parsing with format 3.
- Use `DateTime::parse_from_rfc3339` as a fallback for the full RFC3339 format.
- Both `list_backups` and `prune_backups` call this shared helper instead of inline parsing.
- The parsed `NaiveDateTime` is the UTC time (strip timezone offset if present).

## Files

- `crates/ai-brains-brain/src/backup.rs` — `parse_backup_timestamp` helper + use in `list_backups` and `prune_backups`.

## Tests (TDD)

**Red:** `backup_list__parses_nanosecond_timestamp` — create a backup file named `vault-2026-04-28T16-23-52.639348300+00-00.db.bak` (empty or minimal content), run `list_backups`, assert the timestamp shows `2026-04-28 16:23:52` (not "(unparseable)").

**Red:** `backup_list__parses_nanosecond_no_timezone` — same but with filename `vault-2026-04-28T16-23-52.639348300.db.bak`.

**Red:** `backup_prune__handles_nanosecond_timestamps` — create mixed-format backups, run `prune_backups(2, None, false)`, assert nanosecond-precision backups are correctly sorted and pruned.

**Green:** Implement the multi-format parser. Tests pass.

## Verification

- `cargo nextest run -p ai-brains-brain --lib`
- `cargo nextest run -p ai-brains-cli`
- Manual: `ai-brains backup list` → orphaned backups now show proper timestamps instead of "(unparseable)".

## Out of Scope

- Renaming existing backup files to the current format (too risky, could break user references).
- Changing the current backup naming format (keep `%Y-%m-%dT%H-%M-%S` for new backups).
- Handling non-UTC timezone offsets (all existing backups use `+00:00`).