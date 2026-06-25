# Track T135: `nightly --status` Shows Schedule State

**Status:** Complete
**Started:** —
**Owner:** —
**Priority:** P3 — observability; can't tell if nightly is scheduled without checking schtasks.
**Source:** T119-T132 non-destructive command audit.

---

## Problem Statement

`nightly --status` shows the last run timestamp and pending work, but does NOT indicate whether the nightly task is currently scheduled. A user whose nightly hasn't run in 22 days has no way to know from the status output that scheduling is the problem — they'd need to separately run `schtasks /query` to discover the task doesn't exist.

## Acceptance Criteria

**AC1:** `nightly --status` includes a `Scheduled:` line indicating whether the Windows task is registered:
```
Scheduled: Yes (daily at 01:00)
```
or
```
Scheduled: No (run 'ai-brains nightly --schedule' to enable)
```

**AC2:** The schedule state is determined by querying `schtasks /query /tn "AI-Brains-Nightly"` and checking if it exists. On non-Windows platforms, this line is omitted or shows `Scheduled: (unknown on non-Windows)`.

**AC3:** If the task exists, the start time is extracted from the schtasks query output and shown.

**AC4:** The existing status fields (last run, unsummarized sessions, errors) are unchanged — no regression.

## Design Notes

- **File:** `crates/ai-brains-cli/src/commands/nightly.rs` — in the `--status` branch, before printing the existing status, run `schtasks /query /tn AI-Brains-Nightly /fo CSV /nh` and check if it succeeds.
- **IMPORTANT: Use `/fo CSV` not `/fo LIST`** — `schtasks /fo LIST` output is localized (e.g., Spanish Windows outputs "Próxima hora de ejecución" instead of "Next Run Time"). `/fo CSV` with `/nh` (no header) returns structured data with consistent column order regardless of locale: `TaskName,Next Run Time,Status,Logon Mode,Last Run Time,Last Result,Author,Task To Run`. Column order is stable across locales even though header labels are localized.
- Parse the CSV output: split on commas, the 2nd field (index 1) is the next run time, 3rd field (index 2) is the status ("Ready", "Running", "Disabled", etc.).
- If `schtasks /query` returns a non-zero exit code, the task doesn't exist → `Scheduled: No`.
- On non-Windows: skip the schedule check entirely (`#[cfg(windows)]` guard).

## Files

- `crates/ai-brains-cli/src/commands/nightly.rs` — add schedule state check to `--status` branch.

## Tests (TDD)

**Red:** `nightly_status__shows_schedule_state` — mock or simulate schtasks output, assert "Scheduled:" line appears in status output. This may need to be a unit test of the parsing logic rather than an integration test.

**Green:** Implement the schtasks query + parse. Test passes.

## Verification

- `cargo nextest run -p ai-brains-cli`
- Manual: `ai-brains nightly --status` → shows `Scheduled: Yes (daily at 01:00)`.
- Manual: `ai-brains nightly --unschedule` then `ai-brains nightly --status` → shows `Scheduled: No`.

## Out of Scope

- Scheduling the task from within `--status` (only reports state).
- Showing daemon schedule state (that's `daemon status`).
- Cross-platform scheduling state (Windows-only).