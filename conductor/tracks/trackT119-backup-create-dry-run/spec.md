# Track T119: `backup create --dry-run`

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P3 — consistency; last mutating command without --dry-run.
**Source:** v0.1.1 verification friction #1 + opportunity #3.

---

## Problem Statement

T107 established a unified `--dry-run` pattern for mutating commands (`pin`, `forget`, `ingest`). `backup create` was missed. It is the last mutating command without a `--dry-run` flag:

```
$ ai-brains backup create --dry-run
error: unexpected argument '--dry-run' found
```

While `backup create` is non-destructive (creates a new file, doesn't modify the vault), the inconsistency breaks the user's mental model: every command that writes something should accept `--dry-run` to preview.

## Acceptance Criteria

**AC1:** `backup create --dry-run` prints a preview of what would happen without creating a backup file: `[dry-run] Would create backup at <path>, source vault <vault_path>, estimated size <bytes>.`

**AC2:** `backup create --dry-run` does NOT create any file on disk. No backup file appears in the backup directory.

**AC3:** `backup create --dry-run --keep N` shows the prune preview alongside the backup preview (T104 prune --dry-run already exists; compose the outputs).

**AC4:** `backup create` (without --dry-run) works exactly as before — no regression.

**AC5:** `backup --dry-run` (top-level, no subcommand) also triggers the dry-run preview (since `backup` defaults to `create`).

## Design Notes

- **File:** `crates/ai-brains-cli/src/commands/backup.rs` — `run_create` function.
- Add `dry_run: bool` parameter to `run_create`. When true:
  - Compute the backup path (timestamp + dir) but don't create the file.
  - Get the source vault file size from `fs::metadata(&vault_path)`.
  - Print the preview line.
  - If `--keep N` is also set, call `prune_backups(N, ..., dry_run=true)` and show its preview too.
  - Return `Ok(())` without touching disk.
- **CLI flag:** Add `--dry-run` to the `Backup` subcommand in `main.rs` (or to `BackupCreate` if it's a separate struct). Check how T107 added it to `pin`/`forget`/`ingest` for the pattern.

## Files

- `crates/ai-brains-cli/src/main.rs` — Add `--dry-run` to `backup create` (or `Backup` command).
- `crates/ai-brains-cli/src/commands/backup.rs` — `run_create`: branch on `dry_run`.

## Tests (TDD)

**Red:** `backup_create__dry_run__does_not_create_file` — run `backup create --dry-run`, assert no `.db.bak` file appears in the backup directory.

**Red:** `backup_create__dry_run__prints_preview` — run `backup create --dry-run`, assert stdout contains `[dry-run] Would create backup at` and the vault path.

**Green:** Implement the dry-run branch. Tests pass.

## Verification

- `cargo nextest run -p ai-brains-cli`
- Manual: `ai-brains backup create --dry-run` → preview printed, no file created.
- Manual: `ai-brains backup create` → backup created as before.

## Out of Scope

- Adding `--dry-run` to `backup restore` (already has it from T76).
- Changing the backup file naming convention.
- Backup integrity checking (T131).