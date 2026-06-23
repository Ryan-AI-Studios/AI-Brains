# Track T138: `backup verify` FAIL Error Reason

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P3 — observability; FAIL doesn't distinguish wrong-key from corruption.
**Source:** T119-T132 non-destructive command audit.

---

## Problem Statement

`backup verify` reports `FAIL` for backups that can't be opened, but doesn't show WHY they failed. The 11 FAILs in this vault are all from old backups that predate SQLCipher encryption (or were created with a different key) — they're not corruption, just incompatible. But `backup verify` shows the same `FAIL` for all of them, making it impossible to distinguish:
- Wrong key (harmless — old backup from before encryption)
- Genuine corruption (actionable — need to restore from known-good)
- Missing core tables (partially written backup)

## Acceptance Criteria

**AC1:** `backup verify` text output includes the error reason for FAIL: `vault-2026-05-12.db.bak: FAIL — file is not a database` or `vault-2026-05-12.db.bak: FAIL — missing core tables`.

**AC2:** `backup verify --format json` includes an `error` field in each FAIL result: `{"path": "...", "status": "fail", "error": "file is not a database", ...}`.

**AC3:** OK results are unchanged — no `error` field for passing backups.

**AC4:** The error reason is specific enough to categorize:
- `file is not a database` — wrong key or pre-encryption backup
- `missing core tables` — partially written or empty backup
- `quick_check failed: <detail>` — actual integrity failure
- `unable to open: <io error>` — file system error (permissions, locked)

## Design Notes

- **File:** `crates/ai-brains-cli/src/commands/backup.rs` — `verify_single_backup` already returns `Err(message)`. The `run_verify` function captures this but only sets `status = "fail"` without storing the error message.
- Add an `error: Option<String>` field to `VerifyResult` (with `skip_serializing_if = "Option::is_none"`).
- In `run_verify`, when `verify_single_backup` returns `Err`, store the error message in the `VerifyResult.error` field.
- In the text output, append the error: `FAIL — {error}` (this already works for the `tracing::info!` log line at line 250, but not for the final text output at line 268-276).
- The text output currently just prints `FAIL` — change to `FAIL — {error}` when error is present.

## Files

- `crates/ai-brains-cli/src/commands/backup.rs` — `VerifyResult` struct, `run_verify` function, text output.

## Tests (TDD)

**Red:** `backup_verify__corrupted_backup__shows_error_reason` — create a backup, corrupt it, run `backup verify`, assert output contains `FAIL — ` followed by a specific error reason (not just `FAIL`).

**Red:** `backup_verify__json_includes_error_field` — run `backup verify --format json` on a corrupted backup, parse JSON, assert the fail result has an `error` field with a non-empty value.

**Green:** Store and display the error reason. Tests pass.

## Verification

- `cargo nextest run -p ai-brains-cli`
- Manual: `ai-brains backup verify` → FAIL lines show reason.
- Manual: `ai-brains backup verify --format json` → fail results have `error` field.

## Out of Scope

- Auto-categorizing or grouping failures by type.
- Repairing corrupted backups.
- Skipping known-incompatible backups automatically.