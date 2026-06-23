# Track T116: Backup List Full Paths + Schema Version Fix

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P3 — output cleanliness + auditability. Two related backup display issues.
**Source:** Non-destructive command audit 2026-06-23.

---

## Problem Statement

Two issues with `backup list` and backup metadata:

**Issue 1: Truncated paths.**
`backup list` truncates the path column to 40 characters. Since all backups are in the same directory, the full path is long and gets cut to `C:\dev\ai-brains\backups\vault-2026-0...`, making it impossible to identify or copy-paste the filename. The timestamp column already shows the parsed timestamp, so the path column should show just the filename, not the full path.

**Issue 2: `schema_version: unknown`.**
T109 writes `schema_version` to the backup metadata by trying `SELECT MAX(version) FROM _aibrains_migrations`, then `schema_migrations`, then `migrations`. The actual migration table is `schema_migrations` but it has a `name` column (e.g. `"0006_memory_projection"`), not a `version` column. The query `SELECT MAX(version) FROM schema_migrations` fails because `version` column doesn't exist. Fix: query `SELECT COUNT(*) FROM schema_migrations` or `SELECT MAX(name) FROM schema_migrations` to get the latest migration name.

**Issue 3 (bonus): UNC path in `source_vault_path`.**
`canonicalize()` on Windows produces `\\?\C:\dev\AI-Brains\vault.db`. The `dunce` crate (already in the dependency tree) strips the `\\?\` prefix. Use `dunce::canonicalize()` instead of `std::fs::canonicalize()`.

## Acceptance Criteria

**AC1:** `backup list` shows the backup FILENAME (not full path) in the first column. e.g. `vault-2026-06-23T02-45-51.db.bak` — fully visible, copy-pasteable.

**AC2:** `backup list` column headers update to "Filename" instead of "Path".

**AC3:** New backups (created after this fix) have a correct `schema_version` value in the metadata table — the latest applied migration name (e.g. `"0019_embedding_timestamp"`). See T117 for the query fix.

**AC4:** `backup restore --dry-run` displays the correct schema version for new backups.

**AC5:** `source_vault_path` in the metadata uses a clean Windows path without `\\?\` prefix (e.g. `C:\dev\AI-Brains\vault.db`).

**AC6:** Pre-T109 backups and pre-T116 backups still show `(no metadata)` — no regression.

## Design Notes

- **Issue 1 fix:** In `run_list`, use `info.path.file_name().unwrap_or_default().to_string_lossy()` instead of `info.path.to_string_lossy()`. Remove the `truncate` call for the filename column. Adjust column width to accommodate filenames (~35 chars).

- **Issue 2 fix:** In `backup.rs` (brain crate), change the schema_version query:
  ```rust
  // Old (broken): tries non-existent `version` column
  src_conn.query_row("SELECT MAX(version) FROM schema_migrations", [], |row| row.get(0))

  // New: count applied migrations
  let schema_ver: Option<String> = src_conn
      .query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| {
          let count: i64 = row.get(0)?;
          Ok(count.to_string())
      })
      .ok();
  ```

  Or get the latest migration name:
  ```rust
  let schema_ver: Option<String> = src_conn
      .query_row("SELECT MAX(name) FROM schema_migrations", [], |row| row.get(0))
      .ok();
  ```

  Use `MAX(name)` — it's the high-water mark, safer than `COUNT(*)` for representing schema state. Delegates to T117 for the actual fix.

- **Issue 3 fix:** Add `dunce` as a dependency to `ai-brains-brain` (it's already in the transitive dep tree). Replace `self.vault_path.canonicalize()` with `dunce::canonicalize(&self.vault_path)`.

  Actually, check if `dunce` is accessible. If not directly depended on, add to `crates/ai-brains-brain/Cargo.toml`:
  ```toml
  dunce = "1"
  ```

## Files

- `crates/ai-brains-brain/src/backup.rs` — Fix schema_version query, use `dunce::canonicalize`.
- `crates/ai-brains-brain/Cargo.toml` — Add `dunce` dependency.
- `crates/ai-brains-cli/src/commands/backup.rs` — `run_list`: show filename not full path.

## Tests (TDD)

**Red:** `backup__metadata_has_correct_schema_version` — create a vault with migrations applied, run backup, read metadata, assert `schema_version` is a number string (not "unknown").

**Red:** `backup_list__shows_filename_not_full_path` — create backups, run `backup list`, assert output contains the filename `vault-*.db.bak` not a truncated full path.

**Red:** `backup__metadata_source_path_no_unc_prefix` — run backup, read metadata `source_vault_path`, assert it does NOT start with `\\?\`.

**Green:** Implement fixes. Tests pass.

## Verification

- `cargo nextest run -p ai-brains-brain`
- `cargo nextest run -p ai-brains-cli`
- Manual: `ai-brains backup create` then `ai-brains backup list` → filenames visible, schema version shows count.
- Manual: `ai-brains backup restore --dry-run <new-backup>` → shows `schema_version: 19` (or whatever the count is).

## Out of Scope

- Migrating to a numeric schema version system.
- Adding backup integrity check to `backup list`.
- Changing the metadata table schema.