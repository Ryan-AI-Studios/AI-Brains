# Track T134: `backup list --quiet` Flag

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P3 — UX friction; old/unopenable backups emit WARN noise with no way to suppress.
**Source:** T119-T132 non-destructive command audit.

---

## Problem Statement

`backup list` shows 11 WARN lines for old backups that can't be opened (wrong key, pre-encryption, corrupted). These are expected for any vault with historical backups, but there's no way to suppress them without using `--log-format off` which also suppresses useful INFO output from other commands.

A `--quiet` flag (consistent with `recall --quiet` and `sync query --quiet`) would suppress only the backup-metadata WARN lines while leaving the table output intact.

## Acceptance Criteria

**AC1:** `backup list --quiet` suppresses WARN-level tracing output for backup metadata read failures (expected conditions: old backups, pre-encryption, wrong key). The table output is unchanged.

**AC2:** `backup list --quiet` does NOT suppress genuine I/O errors (backup directory unreadable, permission denied, disk errors). These remain at `warn!` regardless of `--quiet` — the user needs to see them. Only the per-file metadata-read warnings (wrong key, no metadata table, missing core tables) are suppressed.

**AC3:** `backup list` (without `--quiet`) works as before — all WARN lines appear for unopenable backups.

**AC4:** The `--quiet` flag is documented in `backup list --help`.

## Design Notes

- **File:** `crates/ai-brains-cli/src/main.rs` — add `--quiet` to `BackupCommands::List`.
- **File:** `crates/ai-brains-cli/src/commands/backup.rs` — `run_list` accepts `quiet: bool`. When true, the WARN-level tracing calls for metadata read failures are gated on `!quiet` (or use `tracing::debug!` when quiet). The simplest approach: pass `quiet` to `BackupService::list_backups` which conditionally uses `warn!` vs `debug!`.
- Alternatively, since tracing is global, the cleanest approach is to set a thread-local or pass a flag that the brain crate checks. But that's over-engineered — instead, just filter at the CLI level: when `quiet`, don't call `list_backups` with warnings. The simplest fix: add a `quiet` param to `list_backups` in `ai-brains-brain/src/backup.rs` that gates the `warn!` vs `debug!` choice for metadata errors only. Directory I/O errors (the `fs::read_dir` call itself) must remain at `warn!` regardless — they indicate the backup directory is inaccessible, not just an old file.

## Files

- `crates/ai-brains-cli/src/main.rs` — add `--quiet` to `BackupCommands::List`.
- `crates/ai-brains-cli/src/commands/backup.rs` — pass `quiet` to `run_list`.
- `crates/ai-brains-brain/src/backup.rs` — `list_backups` accepts `quiet` param, gates warn vs debug.

## Tests (TDD)

**Red:** `backup_list__quiet__suppresses_metadata_warn` — run `backup list --quiet` with old backups, assert no WARN lines for metadata-read failures on stderr.

**Red:** `backup_list__quiet__preserves_io_error_warn` — make backup directory unreadable (chmod/remove), run `backup list --quiet`, assert WARN for directory I/O error still appears (not suppressed by quiet).

**Green:** Gate metadata warn on `!quiet`, keep I/O warn always. Tests pass.

## Verification

- `cargo nextest run -p ai-brains-cli`
- `cargo nextest run -p ai-brains-brain --lib`
- Manual: `ai-brains backup list --quiet` → clean output, no WARN.

## Out of Scope

- Adding `--quiet` to other backup subcommands (verify, create, prune — each could be a follow-up).
- Changing the backup list table format.