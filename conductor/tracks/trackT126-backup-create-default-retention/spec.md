# Track T126: `backup create` Default Retention

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P2 — backups accumulate indefinitely (20+ in this vault).
**Source:** v0.1.1 verification opportunity #5.

---

## Problem Statement

T104 added `backup prune --keep N` and `backup create --keep N` (auto-prune after backup). However, the default is no pruning. Backups accumulate indefinitely:

```
$ ai-brains backup list
# 20+ backup files, oldest from April 2026
```

Most users don't think about retention until disk space runs out. A sensible default retention policy would prevent unbounded growth without requiring users to remember `--keep`.

## Acceptance Criteria

**AC1:** `backup create` defaults to `--keep 10` (keeps the 10 most recent backups, prunes older ones) when `--keep` is not explicitly provided.

**AC2:** `backup create --keep 0` or `backup create --no-prune` disables pruning entirely (keeps all backups). This is the opt-out for users who want to keep everything.

**AC3:** `backup create --keep N` (explicit) overrides the default with N.

**AC4:** When default pruning runs, it prints a brief summary: `Pruned 3 old backups (kept 10).` via `tracing::info!`.

**AC5:** The most recent backup is ALWAYS preserved, even if `--keep 1` is set.

**AC6:** No regression in `backup prune` standalone command (it still works with explicit `--keep`).

## Design Notes

- **File:** `crates/ai-brains-cli/src/commands/backup.rs` — `run_create` function.
- Change the `keep` parameter default from `None` (no pruning) to `Some(10)`.
- Add a `--no-prune` flag that sets `keep = None`.
- The existing prune logic in T104 already handles `keep=0` as "keep all" — verify this or make `--keep 0` explicitly disable pruning.
- The pruning should happen AFTER the backup is created (so the new backup is included in the count and the oldest is pruned).
- Print the prune result via `tracing::info!` only if backups were actually pruned (don't print "Pruned 0" noise).

## Files

- `crates/ai-brains-cli/src/main.rs` — Add `--no-prune` flag, change `--keep` default.
- `crates/ai-brains-cli/src/commands/backup.rs` — `run_create`: apply default retention.

## Tests (TDD)

**Red:** `backup_create__default_keep_10__prunes_old_backups` — create 12 backups, then run `backup create` (no --keep), assert only 10 remain.

**Red:** `backup_create__no_prune__keeps_all` — create several backups, run `backup create --no-prune`, assert all backups remain.

**Red:** `backup_create__explicit_keep_N__overrides_default` — run `backup create --keep 3`, assert only 3 remain.

**Green:** Implement default retention. Tests pass.

## Verification

- `cargo nextest run -p ai-brains-cli`
- `cargo nextest run -p ai-brains-brain --lib`
- Manual: `ai-brains backup create` → creates backup + prunes to 10.
- Manual: `ai-brains backup create --no-prune` → creates backup, no pruning.

## Out of Scope

- Configurable default retention via config file or env var (could be future track).
- Backup size-based retention (keep backups under X total GB).
- Time-based default retention (keep backups from last 30 days).