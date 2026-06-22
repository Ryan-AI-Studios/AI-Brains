# Track T109: Backup Metadata Header Table

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P3 — auditability; backups have no provenance metadata.
**Source:** Systematic command test 2026-06-22.

---

## Problem Statement

Backup files are raw SQLite DB copies. There is no metadata indicating when the backup was created, what vault path it came from, what schema version it was, or what AI-Brains version created it. When restoring, users can't verify the backup's origin without manually querying it. A metadata table inside the backup would make backups self-describing.

## Acceptance Criteria

**AC1:** After the SQLite backup API completes, `run_backup` creates a `_aibrains_backup_meta` table in the backup file with: `key TEXT PRIMARY KEY, value TEXT`.

**AC2:** The metadata table includes at minimum: `backup_timestamp` (ISO 8601), `source_vault_path` (absolute path), `ai_brains_version` (from `CARGO_PKG_VERSION`), `schema_version` (read from the vault's migration table), `backup_file_size_bytes`.

**AC3:** `backup restore --dry-run` reads and displays the metadata: `Backup created: <timestamp>, Source: <path>, Version: <ver>, Schema: <ver>`.

**AC4:** `backup list` (new subcommand) lists all backups in the backup directory with their metadata in a table format. If a backup doesn't have the metadata table (pre-T109 backups), it shows `(no metadata)` for those fields.

**AC5:** The metadata table does not interfere with the restore process — it's ignored by the SQLite backup API on restore (the table is restored along with everything else, but the restore overwrites the vault including any existing metadata table).

## Design Notes

- The metadata table is prefixed with `_aibrains_` to avoid collision with user tables and make it clearly AI-Brains-internal.
- Write the metadata AFTER the backup API completes (so the backup is already a valid copy), using a simple `CREATE TABLE` + `INSERT` on the destination connection.
- `schema_version`: read from `cozo_meta` or the AI-Brains migrations table (check what table tracks the current migration version).
- `backup list`: scan the backup directory for `vault-*.db.bak` files, open each, try to read `_aibrains_backup_meta`, display in a table.
- Keep it simple — no JSON, no nested values, just key-value pairs.

## Files

- `crates/ai-brains-brain/src/backup.rs` — write metadata table after backup, add `list_backups` method.
- `crates/ai-brains-cli/src/commands/backup.rs` — display metadata in `--dry-run`, add `run_list` function.
- `crates/ai-brains-cli/src/main.rs` — add `List` variant to `BackupCommands`.

## Tests (TDD)

**Red:** `backup__creates_metadata_table` — run backup, open the backup file, assert `_aibrains_backup_meta` table exists with `backup_timestamp` and `source_vault_path` keys.

**Green:** Add metadata write. Test passes.

## Verification

- `cargo nextest run -p ai-brains-brain`
- `cargo nextest run -p ai-brains-cli`
- Manual: `ai-brains backup create` then `ai-brains backup list` → shows metadata table.
- Manual: `ai-brains backup restore <path> --dry-run` → displays metadata.

## Out of Scope

- Backup integrity verification on `backup list` (just read metadata, don't run integrity_check).
- Remote backup metadata.
- Backup diff/comparison.
