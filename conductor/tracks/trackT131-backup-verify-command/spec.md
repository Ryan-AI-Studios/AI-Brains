# Track T131: `backup verify` Command

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P3 — safety; no way to detect silent backup corruption before restore.
**Source:** v0.1.1 verification opportunity #10.

---

## Problem Statement

`ai-brains backup list` shows metadata (timestamp, size, schema version) but doesn't verify backup integrity. `ai-brains backup restore --dry-run` checks the specific backup being restored, but there's no way to check ALL backups at once. Silent corruption (disk errors, partial writes, encryption key mismatch) would only be discovered at restore time — too late.

## Acceptance Criteria

**AC1:** `backup verify` (no path argument) verifies ALL backup files in the backup directory. For each backup:
  - Open with the vault key.
  - Run `PRAGMA integrity_check` — must return `ok`.
  - Verify the backup contains expected tables (at minimum: `events` or `memory_projection` — the core schema).
  - Report: `vault-2026-06-23T14-50-09.db.bak: OK` or `vault-2026-06-23T14-50-09.db.bak: FAIL — <error>`

**AC2:** `backup verify <path>` verifies a single backup file.

**AC3:** `backup verify --format json` outputs structured JSON:
```json
{"results": [{"path": "...", "status": "ok", "integrity_check": "ok", "tables": ["events", "memory_projection"], "size_bytes": 14012416}]}
```

**AC4:** `backup verify` exits with code 1 if ANY backup fails verification, 0 if all pass.

**AC5:** Backups that can't be opened (wrong key, corrupted header) are reported as `FAIL — <error>` not crashed on.

**AC6:** Pre-T109 backups (no metadata table) still verify — the integrity check and core table check don't depend on the metadata table.

**AC7:** `backup verify --dry-run` is NOT needed (verify is already non-destructive — it only reads).

## Design Notes

- **File:** `crates/ai-brains-cli/src/commands/backup.rs` — new `run_verify` function.
- **File:** `crates/ai-brains-cli/src/main.rs` — add `Verify` subcommand to `Backup`.
- For each backup file:
  1. `rusqlite::Connection::open(path)` + `apply_key_pragmas`.
  2. `PRAGMA integrity_check` → expect `ok`.
  3. Check for core tables: `SELECT name FROM sqlite_master WHERE type='table' AND name IN ('events', 'memory_projection')`.
  4. Report result.
- Use `tracing::info!` for progress: `Verifying N backup files...`
- Handle errors gracefully: if a backup can't be opened, report `FAIL` with the error message, continue to the next backup.
- The `BackupService` already has `read_backup_metadata` — reuse for metadata display if needed.

## Files

- `crates/ai-brains-cli/src/main.rs` — Add `backup verify` subcommand.
- `crates/ai-brains-cli/src/commands/backup.rs` — `run_verify` function.

## Tests (TDD)

**Red:** `backup_verify__valid_backup__reports_ok` — create a backup, run `backup verify <path>`, assert output contains `OK` and exit 0.

**Red:** `backup_verify__corrupted_backup__reports_fail` — create a backup, corrupt it (append random bytes), run `backup verify <path>`, assert output contains `FAIL` and exit 1.

**Red:** `backup_verify_all__mixed__reports_per_file` — create 2 valid + 1 corrupted backup, run `backup verify` (no path), assert 2 `OK` + 1 `FAIL`, exit 1.

**Red:** `backup_verify__json_format` — run `backup verify --format json`, parse JSON, assert structured results.

**Green:** Implement `run_verify`. Tests pass.

## Verification

- `cargo nextest run -p ai-brains-cli`
- Manual: `ai-brains backup verify` → all backups checked, summary printed.
- Manual: `ai-brains backup verify <path>` → single backup checked.

## Out of Scope

- Repairing corrupted backups (out of scope — restore from a known-good backup instead).
- Verifying backup content against the live vault (data consistency check, not integrity check).
- Scheduling periodic verification (that's an operational concern, not a CLI feature).
- Verifying backups on remote storage (local files only for now).