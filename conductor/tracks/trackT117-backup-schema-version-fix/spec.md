# Track T117: Backup Schema Version Fix

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P3 — auditability; metadata shows `unknown` instead of the actual migration count.
**Source:** Non-destructive command audit 2026-06-23. Split from T116 for focused scope.

---

## Problem Statement

T109 backup metadata writes `schema_version` by querying `SELECT MAX(version) FROM _aibrains_migrations`, then `SELECT MAX(version) FROM schema_migrations`, then `SELECT MAX(version) FROM migrations`. All three fail because:

1. `_aibrains_migrations` doesn't exist — the table is named `schema_migrations`.
2. `schema_migrations` exists but has a `name` column (e.g. `"0019_embedding_timestamp"`), NOT a `version` column. `SELECT MAX(version)` fails with "no such column: version".
3. `migrations` doesn't exist.

Result: `schema_version` is always `"unknown"` in backup metadata.

## Acceptance Criteria

**AC1:** New backups have `schema_version` set to the count of applied migrations as a string (e.g. `"19"`).

**AC2:** `backup restore --dry-run` displays the correct schema version.

**AC3:** `backup list` displays the correct schema version in the metadata column (if a schema_version column is shown).

## Design Notes

- **Fix in `crates/ai-brains-brain/src/backup.rs`:** Replace the schema_ver query:
  ```rust
  let schema_ver: Option<String> = src_conn
      .query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| {
          let count: i64 = row.get(0)?;
          Ok(count.to_string())
      })
      .ok();
  ```

- The migration table is `schema_migrations` with `name TEXT PRIMARY KEY` and `applied_at TEXT`. No `version` column. Count of rows = number of applied migrations = schema version.

## Files

- `crates/ai-brains-brain/src/backup.rs` — Fix the schema_ver query (around line 120-130 in the T109 metadata section).

## Tests (TDD)

**Red:** `backup__metadata_has_correct_schema_version` — create a vault, apply migrations, run backup, read `_aibrains_backup_meta`, assert `schema_version` is a parseable integer string (not "unknown").

**Green:** Fix the query. Test passes.

## Verification

- `cargo nextest run -p ai-brains-brain`
- Manual: `ai-brains backup create` then `ai-brains backup restore --dry-run <path>` → shows `schema_version: 19`.

## Out of Scope

- Renaming the migration table.
- Adding a numeric version column.
- UNC path fix (covered by T116).