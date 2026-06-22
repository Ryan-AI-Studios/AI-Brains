# Track T103: Add `--dry-run` to `daemon schedule`

**Status:** ✅ Complete
**Started:** 2026-06-22
**Owner:** Claude
**Priority:** P3 — UX polish; lets users preview the schtasks command without registering it.
**Source:** Non-destructive command test 2026-06-22.

---

## Problem Statement

`ai-brains daemon schedule` registers a Windows Task Scheduler logon task to auto-start the daemon. There is no way to preview what it will do without actually registering the task. The nightly scheduler (`ledgerful schedule setup-nightly`) has `--dry-run`, but `daemon schedule` does not.

## Acceptance Criteria

**AC1:** `ai-brains daemon schedule --dry-run` prints the schtasks command that would be executed without actually running it.

**AC2:** `--dry-run` does not create any Task Scheduler entries or modify the system.

**AC3:** The output includes the full schtasks command, the daemon logon command, and the path to the daemon executable.

**AC4:** Without `--dry-run`, the behavior is unchanged (registers the task as before).

**AC5:** `daemon unschedule --dry-run` is also added (symmetry). Prints the schtasks /delete command without executing it.

**AC6:** The dry-run output includes a note: `(Note: actual registration may require elevated PowerShell privileges depending on system policy)` so users are warned about UAC before running for real.

## Design Notes

- Add a `--dry-run` flag to the `DaemonSchedule` variant in `crates/ai-brains-cli/src/main.rs`.
- In `crates/ai-brains-cli/src/commands/daemon.rs`, the `schedule` function builds and executes a schtasks command. Add a `dry_run: bool` parameter. When true, print the command instead of executing it.
- The schtasks quoting logic (T78) already uses `render_daemon_logon_command` — the dry-run should print both the rendered command and the full schtasks invocation.

## Files

- `crates/ai-brains-cli/src/main.rs` — add `--dry-run` to `DaemonSchedule` and `DaemonUnschedule`.
- `crates/ai-brains-cli/src/commands/daemon.rs` — add `dry_run` parameter to both schedule and unschedule functions.

## Tests (TDD)

**Red:** `daemon_schedule_dry_run__prints_command_without_registering` — run `daemon schedule --dry-run`, assert stdout contains "schtasks" and no task is created (check via `schtasks /query /tn AI-BrainsDaemon` before and after — should not exist).

**Green:** Add `--dry-run` flag. Test passes.

## Verification

- `cargo nextest run -p ai-brains-cli`
- Manual: `ai-brains daemon schedule --dry-run` — prints the schtasks command, does not create a task.
- Manual: `ai-brains daemon schedule` — creates the task (existing behavior).

## Out of Scope

- Changes to the nightly scheduler's `--dry-run` (already exists).
- Modifying the schtasks quoting logic.