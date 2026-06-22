# Track T104: Backup Retention / Prune Policy

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P2 — backups accumulate indefinitely; disk bloat is inevitable without cleanup.
**Source:** Systematic command test 2026-06-22.

---

## Problem Statement

`ai-brains backup create` generates timestamped backup files in `backups/vault-YYYY-MM-DDTHH-MM-SS.db.bak`. There is no retention policy or cleanup mechanism. After weeks of daily backups, the directory grows unbounded. Users must manually delete old backups. There is no `backup prune` command, no `--keep` flag on `backup create`, and no max-age enforcement.

## Acceptance Criteria

**AC1:** A new `ai-brains backup prune` subcommand is added that deletes old backup files according to a retention policy.

**AC2:** `backup prune` accepts `--keep <N>` (keep the N most recent backups, default 10) and `--older-than <duration>` (e.g. `30d`, `12h`, delete backups older than this). Both can be combined — a backup is deleted if it fails BOTH criteria (older than the threshold AND beyond the keep count).

**AC3:** `backup prune --dry-run` lists the files that would be deleted without actually deleting them.

**AC4:** `backup create` accepts an optional `--keep <N>` flag that runs `prune` after a successful backup, keeping only the N most recent backups (including the new one).

**AC5:** `backup prune` outputs a summary: `Pruned N backup(s), K remaining. Freed X MB.`

**AC6:** `backup prune` only deletes files matching the `vault-*.db.bak` pattern in the backup directory. It never touches other files.

## Design Notes

- The backup directory is `vault_path.parent()/backups/` (or the custom `--output-dir`).
- List files matching `vault-*.db.bak`, sort by timestamp (parsed from filename, not mtime — mtime is unreliable on copied files), then apply the retention policy.
- Duration parsing: support `Nd` (days), `Nh` (hours), `Nw` (weeks). Use `humantime` crate if available, or a simple parser.
- The `--keep` count includes the newly created backup when called via `backup create --keep N`.
- Always keep at least 1 backup (refuse to prune the most recent file even if `--keep 0`).

## Files

- `crates/ai-brains-brain/src/backup.rs` — add `prune_backups` method to `BackupService`.
- `crates/ai-brains-cli/src/commands/backup.rs` — add `run_prune` function.
- `crates/ai-brains-cli/src/main.rs` — add `Prune` variant to `BackupCommands`.

## Tests (TDD)

**Red:** `backup_prune__keep_2__deletes_oldest` — create 5 backup files with mock timestamps, run `prune --keep 2`, assert only 2 newest remain.

**Green:** Implement prune. Test passes.

## Verification

- `cargo nextest run -p ai-brains-brain`
- `cargo nextest run -p ai-brains-cli`
- Manual: `ai-brains backup prune --dry-run --keep 5` lists files correctly.

## Out of Scope

- Backup compression.
- Remote backup storage (S3, etc.).
- Backup verification on prune (only delete, don't verify).
