# Track T132: `--run-as-system` Flag for Schedule Commands

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P3 — UX friction; scheduled tasks only run when logged in.
**Source:** T119-T131 verification friction.

---

## Problem Statement

`ai-brains nightly --schedule` and `ai-brains daemon --schedule` register Windows scheduled tasks using `schtasks /create` without the `/ru` flag, which defaults to the current user in **Interactive only** logon mode. This means:

- Tasks do NOT run if no one is logged in.
- Users must manually re-register the task from an elevated shell with `/ru SYSTEM` to run unattended.
- The CLI provides no way to request this behavior.

This is a friction point for anyone who wants nightly sweeps or daemon auto-start to work reliably regardless of login state.

## Acceptance Criteria

**AC1:** `nightly --schedule --run-as-system` registers the scheduled task with `/ru SYSTEM` instead of the default interactive user. The task runs at the specified time regardless of whether anyone is logged in.

**AC2:** `daemon --schedule --run-as-system` does the same for the daemon auto-start task.

**AC3:** When `--run-as-system` is NOT passed, behavior is unchanged (current user, interactive only) — no regression.

**AC4:** `nightly --schedule --run-as-system --dry-run` (if dry-run exists) prints the schtasks command with `/ru SYSTEM` so users can verify before registering.

**AC5:** When `--run-as-system` is used and the user is NOT elevated, the error message clearly states: `Scheduling as SYSTEM requires elevation. Re-run from an Administrator shell.`

**AC6:** The `--run-as-system` flag is documented in `--help` output for both `nightly` and `daemon schedule`.

## Design Notes

- **Files:**
  - `crates/ai-brains-cli/src/main.rs` — add `#[arg(long)] run_as_system: bool` to `Nightly` command and `DaemonCommands::Schedule`.
  - `crates/ai-brains-cli/src/commands/nightly.rs` — pass `run_as_system` to the schtasks args.
  - `crates/ai-brains-cli/src/commands/daemon.rs` — same for `run_schedule`.
- When `run_as_system` is true, append `/ru SYSTEM` to the schtasks `/create` args.
- The `/ru SYSTEM` flag tells Windows to run the task as the LocalSystem account, which does not require a login session.
- Detect elevation failure from schtasks exit code/stderr and return a clear error.

## Files

- `crates/ai-brains-cli/src/main.rs` — add `--run-as-system` flag.
- `crates/ai-brains-cli/src/commands/nightly.rs` — pass flag to schtasks.
- `crates/ai-brains-cli/src/commands/daemon.rs` — pass flag to schtasks.

## Tests (TDD)

**Red:** `nightly_schedule__run_as_system__adds_ru_system` — unit test that the rendered schtasks command includes `/ru SYSTEM` when `run_as_system` is true.

**Red:** `nightly_schedule__no_run_as_system__omits_ru_system` — unit test that `/ru SYSTEM` is absent by default.

**Red:** `nightly_schedule__run_as_system_not_elevated__clear_error` — test that the error message mentions "elevation" when schtasks returns access denied.

**Green:** Implement the flag. Tests pass.

## Verification

- `cargo nextest run -p ai-brains-cli`
- Manual (elevated): `ai-brains nightly --schedule --run-as-system --start-time 01:00` → task registered as SYSTEM.
- `schtasks /query /tn AI-Brains-Nightly /fo LIST` → `Run As User: SYSTEM`.

## Out of Scope

- Cross-platform scheduling (Linux cron, macOS launchd) — Windows-only for now.
- Storing encrypted credentials for a specific user account (SYSTEM is sufficient for local-first).
- Auto-detecting elevation state before attempting schtasks (let schtasks report the error).